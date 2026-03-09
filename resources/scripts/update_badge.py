import json
import os
import urllib.parse
import urllib.request

USER_AGENT = "CopyPaste-Badge-Updater"

WINDOWS_EXTENSIONS = (".exe", ".msix", ".msixbundle")
MACOS_EXTENSIONS = (".dmg",)


def get_gh_downloads_by_os(repo):
    url = f"https://api.github.com/repos/{repo}/releases"
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    windows = 0
    macos = 0
    other = 0
    try:
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read())
        for release in data:
            for asset in release.get("assets", []):
                name = asset.get("name", "").lower()
                count = asset.get("download_count", 0)
                if any(name.endswith(ext) for ext in WINDOWS_EXTENSIONS):
                    windows += count
                elif any(name.endswith(ext) for ext in MACOS_EXTENSIONS):
                    macos += count
                else:
                    other += count
    except Exception as e:
        print(f"Warning: Failed to get GitHub downloads: {e}")
    return windows, macos, other

def get_ms_token(tenant, client_id, client_secret):
    url = f"https://login.microsoftonline.com/{tenant}/oauth2/token"
    payload = urllib.parse.urlencode({
        'grant_type': 'client_credentials',
        'resource': 'https://manage.devcenter.microsoft.com',
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
    url = f"https://manage.devcenter.microsoft.com/v1.0/my/analytics/acquisitions?applicationId={app_id}&aggregationLevel=day"
    req = urllib.request.Request(url, headers={
        'Authorization': f'Bearer {token}',
        'User-Agent': USER_AGENT
    })
    try:
        with urllib.request.urlopen(req) as response:
            data = json.loads(response.read())
        return sum(item.get('acquisitionQuantity', 0) for item in data.get('Value', []))
    except Exception as e:
        print(f"Warning: Failed to get MS Store downloads: {e}")
        return 0

def format_count(n):
    if n >= 1_000_000:
        return f"{n / 1_000_000:.1f}M"
    if n >= 1_000:
        return f"{n / 1_000:.1f}k"
    return str(n)


def update_gist(gist_id, token, windows_total, macos_total, grand_total):
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

    gh_windows, gh_macos, gh_other = get_gh_downloads_by_os(repo)
    print(f"GitHub downloads — Windows: {gh_windows}, macOS: {gh_macos}, Other: {gh_other}")

    ms_total = 0
    if all([tenant_id, client_id, client_secret, app_id]):
        try:
            ms_token = get_ms_token(tenant_id, client_id, client_secret)
            ms_total = get_ms_downloads(ms_token, app_id)
            print(f"MS Store downloads: {ms_total}")
        except Exception as e:
            print(f"Warning: MS Store auth failed: {e}")
    else:
        print("MS Store credentials not configured, skipping")

    windows_total = gh_windows + ms_total
    macos_total = gh_macos
    grand_total = windows_total + macos_total + gh_other

    print(f"Windows total: {windows_total} (GitHub: {gh_windows} + Store: {ms_total})")
    print(f"macOS total: {macos_total}")
    print(f"Grand total: {grand_total}")

    if gist_id and gist_token:
        try:
            update_gist(gist_id, gist_token, windows_total, macos_total, grand_total)
            update_gist(gist_id, gist_token, total)
            print("Badge updated successfully")
        except Exception as e:
            print(f"Error updating gist: {e}")
            raise
    else:
        print("Gist credentials not configured, skipping badge update")

if __name__ == "__main__":
    main()
