import json
import os
import time
import urllib.error
import urllib.parse
import urllib.request
from datetime import date

USER_AGENT = "CopyPaste-Badge-Updater"

WINDOWS_EXTENSIONS = (".exe", ".msix", ".msixupload", ".msixbundle")
MACOS_EXTENSIONS = (".dmg",)
LINUX_EXTENSIONS = (".appimage", ".deb", ".rpm", ".tar.gz")

CLOUDSMITH_METRICS_URL = "https://api.cloudsmith.io/v1/metrics/packages/{owner}/{repo}/"
MS_MAX_RETRIES = 4
MS_PAGE_SIZE = 10000
MS_DEFAULT_START_DATE = "01/02/2026"


def describe_http_error(error):
    body_text = ""
    body_json = None
    try:
        raw = error.read()
        if raw:
            body_text = raw.decode("utf-8", errors="replace").strip()
            try:
                body_json = json.loads(body_text)
            except json.JSONDecodeError:
                body_json = None
    except Exception:
        body_text = ""
        body_json = None

    return {
        "status": error.code,
        "reason": getattr(error, "reason", ""),
        "url": getattr(error, "url", ""),
        "retry_after": error.headers.get("Retry-After"),
        "ms_request_id": error.headers.get("x-ms-request-id"),
        "ms_correlation_id": error.headers.get("x-ms-correlation-id"),
        "body_text": body_text,
        "body_json": body_json,
    }


def get_github_headers():
    headers = {
        "User-Agent": USER_AGENT,
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
    }
    github_token = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    if github_token:
        headers["Authorization"] = f"Bearer {github_token}"
    return headers


def get_gh_downloads_by_os(repo):
    windows = 0
    macos = 0
    linux = 0
    other = 0
    page = 1
    try:
        while True:
            url = f"https://api.github.com/repos/{repo}/releases?per_page=150&page={page}"
            req = urllib.request.Request(url, headers=get_github_headers())
            with urllib.request.urlopen(req) as response:
                data = json.loads(response.read())
            if not data:
                break
            for release in data:
                for asset in release.get("assets", []):
                    name = asset.get("name", "").lower()
                    count = asset.get("download_count", 0)
                    if any(name.endswith(ext) for ext in WINDOWS_EXTENSIONS):
                        windows += count
                    elif any(name.endswith(ext) for ext in MACOS_EXTENSIONS):
                        macos += count
                    elif any(name.endswith(ext) for ext in LINUX_EXTENSIONS):
                        linux += count
                    else:
                        other += count
            if len(data) < 150:
                break
            page += 1
    except urllib.error.HTTPError as e:
        if e.code == 403:
            remaining = e.headers.get("X-RateLimit-Remaining")
            reset_at = e.headers.get("X-RateLimit-Reset")
            print(
                "Warning: Failed to get GitHub downloads: HTTP 403 rate limit exceeded. "
                f"remaining={remaining} reset={reset_at}. "
                "Configure GITHUB_TOKEN in workflow env to use authenticated requests."
            )
        else:
            print(f"Warning: Failed to get GitHub downloads: HTTP Error {e.code}: {e.reason}")
    except Exception as e:
        print(f"Warning: Failed to get GitHub downloads: {e}")
    return windows, macos, linux, other

def get_cloudsmith_downloads(api_key, owner, repo):
    url = CLOUDSMITH_METRICS_URL.format(owner=owner, repo=repo)
    req = urllib.request.Request(url, headers={
        "X-Api-Key": api_key,
        "User-Agent": USER_AGENT
    })
    try:
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read())
        return data.get("packages", {}).get("downloads", {}).get("total", {}).get("value", 0)
    except Exception as e:
        print(f"Warning: Failed to get Cloudsmith downloads: {e}")
        return 0


def get_ms_token(tenant, client_id, client_secret):
    url = f"https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token"
    payload = urllib.parse.urlencode({
        'grant_type': 'client_credentials',
        'scope': 'https://manage.devcenter.microsoft.com/.default',
        'client_id': client_id,
        'client_secret': client_secret
    }).encode('utf-8')
    req = urllib.request.Request(url, data=payload, headers={
        "Content-Type": "application/x-www-form-urlencoded",
        "User-Agent": USER_AGENT
    })
    with urllib.request.urlopen(req) as response:
        return json.loads(response.read())['access_token']

def get_ms_downloads(token, app_id):
    base_url = "https://manage.devcenter.microsoft.com/v1.0/my/analytics/appacquisitions"
    start_date = os.environ.get("STORE_START_DATE", MS_DEFAULT_START_DATE)
    today = date.today()
    end_date = f"{today.month}/{today.day}/{today.year}"

    params = {
        "applicationId": app_id,
        "startDate": start_date,
        "endDate": end_date,
        "aggregationLevel": "day",
        "top": MS_PAGE_SIZE,
        "skip": 0,
    }

    next_url = f"{base_url}?{urllib.parse.urlencode(params)}"
    total = 0
    page_count = 0
    max_pages = 20
    last_error = None
    last_error_details = None

    while next_url and page_count < max_pages:
        page_count += 1
        request_failed = False
        error_details = None

        for attempt in range(1, MS_MAX_RETRIES + 1):
            req = urllib.request.Request(next_url, headers={
                "Authorization": f"Bearer {token}",
                "User-Agent": USER_AGENT
            })
            try:
                with urllib.request.urlopen(req) as response:
                    data = json.loads(response.read())
                break
            except urllib.error.HTTPError as e:
                last_error = e
                error_details = describe_http_error(e)
                last_error_details = error_details
                if e.code == 429 and attempt < MS_MAX_RETRIES:
                    retry_after = error_details.get("retry_after")
                    if retry_after and retry_after.isdigit():
                        wait_seconds = int(retry_after)
                    else:
                        wait_seconds = min(2 ** attempt, 60)
                    print(f"Warning: MS Store throttled (429). Retrying in {wait_seconds}s ({attempt}/{MS_MAX_RETRIES})")
                    time.sleep(wait_seconds)
                    continue
                request_failed = True
                break
            except Exception as e:
                last_error = e
                request_failed = True
                break

        if request_failed:
            if error_details is not None:
                print(
                    "MS Store error details: "
                    f"status={error_details['status']} reason={error_details['reason']} "
                    f"url={error_details['url']} "
                    f"request_id={error_details['ms_request_id']} "
                    f"correlation_id={error_details['ms_correlation_id']}"
                )
                if error_details["body_json"] is not None:
                    print(f"MS Store error body: {json.dumps(error_details['body_json'], ensure_ascii=False)}")
                elif error_details["body_text"]:
                    print(f"MS Store error body: {error_details['body_text']}")
            break

        rows = data.get("Value")
        if rows is None:
            rows = data.get("value", [])
        total += sum(item.get("acquisitionQuantity", 0) for item in rows)

        next_url = data.get("@nextLink") or data.get("nextLink")

    if next_url and page_count >= max_pages:
        print("Warning: MS Store response pagination exceeded limit; stopping at 20 pages")

    if last_error is None:
        return total

    if isinstance(last_error, urllib.error.HTTPError) and last_error.code == 404:
        if (
            last_error_details is not None
            and "Could not find user" in (last_error_details.get("body_text") or "")
        ):
            print(
                "Warning: Failed to get MS Store downloads: the Microsoft Entra application/service principal "
                "was not found in Partner Center user policy. Add the app under Partner Center Users "
                "(Microsoft Entra applications) and grant Manager role in the same tenant as STORE_TENANT_ID."
            )
            return None
        print(
            "Warning: Failed to get MS Store downloads: HTTP 404 from appacquisitions endpoint. "
            "Check STORE_APP_ID and Partner Center app permissions."
        )
    elif isinstance(last_error, urllib.error.HTTPError) and last_error.code == 429:
        print(
            "Warning: Failed to get MS Store downloads: HTTP 429 Too Many Requests after retries. "
            "Skipping Store contribution for this run."
        )
    elif last_error is not None:
        print(f"Warning: Failed to get MS Store downloads: {last_error}")

    return None

def format_count(n):
    if n >= 1_000_000:
        return f"{n / 1_000_000:.1f}M"
    if n >= 1_000:
        return f"{n / 1_000:.1f}k"
    return str(n)


def update_gist(gist_id, token, windows_total, macos_total, linux_total, grand_total):
    url = f"https://api.github.com/gists/{gist_id}"
    payload = {
        "files": {
            "copypaste_downloads.json": {
                "content": json.dumps({
                    "schemaVersion": 1,
                    "label": "downloads",
                    "message": format_count(grand_total),
                    "color": "0078D7"
                })
            },
            "copypaste_downloads_windows.json": {
                "content": json.dumps({
                    "schemaVersion": 1,
                    "label": "Windows downloads",
                    "message": format_count(windows_total),
                    "color": "0078D4",
                    "namedLogo": "windows",
                    "logoColor": "white"
                })
            },
            "copypaste_downloads_macos.json": {
                "content": json.dumps({
                    "schemaVersion": 1,
                    "label": "macOS downloads",
                    "message": format_count(macos_total),
                    "color": "333333",
                    "namedLogo": "apple",
                    "logoColor": "white"
                })
            },
            "copypaste_downloads_linux.json": {
                "content": json.dumps({
                    "schemaVersion": 1,
                    "label": "Linux downloads",
                    "message": format_count(linux_total),
                    "color": "FCC624",
                    "namedLogo": "linux",
                    "logoColor": "black"
                })
            }
        }
    }
    req = urllib.request.Request(url, method="PATCH", data=json.dumps(payload).encode('utf-8'), headers={
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3+json",
        "User-Agent": USER_AGENT,
        "Content-Type": "application/json"
    })
    urllib.request.urlopen(req)

def main():
    repo = os.environ.get("GITHUB_REPOSITORY", "rgdevment/CopyPaste")
    gist_id = os.environ.get("GIST_ID")
    gist_token = os.environ.get("GIST_TOKEN")
    tenant_id = os.environ.get("STORE_TENANT_ID")
    client_id = os.environ.get("STORE_CLIENT_ID")
    client_secret = os.environ.get("STORE_CLIENT_SECRET")
    app_id = os.environ.get("STORE_APP_ID")
    cloudsmith_api_key = os.environ.get("CLOUDSMITH_API_KEY")

    gh_windows, gh_macos, gh_linux, gh_other = get_gh_downloads_by_os(repo)
    print(f"GitHub downloads — Windows: {gh_windows}, macOS: {gh_macos}, Linux: {gh_linux}, Other: {gh_other}")

    ms_total = 0
    ms_fetch_ok = True
    ms_enabled = all([tenant_id, client_id, client_secret, app_id])
    if ms_enabled:
        try:
            ms_token = get_ms_token(tenant_id, client_id, client_secret)
            ms_value = get_ms_downloads(ms_token, app_id)
            if ms_value is None:
                ms_fetch_ok = False
                ms_total = 0
            else:
                ms_total = ms_value
                print(f"MS Store downloads: {ms_total}")
        except Exception as e:
            ms_fetch_ok = False
            print(f"Warning: MS Store auth failed: {e}")
    else:
        print("MS Store credentials not configured, skipping")

    cloudsmith_linux = 0
    if cloudsmith_api_key:
        cloudsmith_linux = get_cloudsmith_downloads(cloudsmith_api_key, "rgdevment", "copypaste")
        print(f"Cloudsmith downloads: {cloudsmith_linux}")
    else:
        print("Cloudsmith API key not configured, skipping")

    windows_total = gh_windows + ms_total
    macos_total = gh_macos
    linux_total = gh_linux + cloudsmith_linux
    grand_total = windows_total + macos_total + linux_total + gh_other

    print(f"Windows total: {windows_total} (GitHub: {gh_windows} + Store: {ms_total})")
    print(f"macOS total: {macos_total}")
    print(f"Linux total: {linux_total} (GitHub: {gh_linux} + Cloudsmith: {cloudsmith_linux})")
    print(f"Grand total: {grand_total}")

    if ms_enabled and not ms_fetch_ok:
        print("Warning: Skipping badge update to avoid publishing partial totals due MS Store fetch failure")
        return

    if gist_id and gist_token:
        try:
            update_gist(gist_id, gist_token, windows_total, macos_total, linux_total, grand_total)
            print("Badge updated successfully")
        except Exception as e:
            print(f"Error updating gist: {e}")
            raise
    else:
        print("Gist credentials not configured, skipping badge update")

if __name__ == "__main__":
    main()
