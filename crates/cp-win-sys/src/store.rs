use windows::ApplicationModel::{Package, PackageSignatureKind, PackageVersion};
use windows::Management::Deployment::PackageManager;
use windows::Services::Store::{
    StoreContext, StorePackageUpdateState, StorePackageUpdateStatus, StoreQueueItemExtendedState,
    StoreQueueItemState,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::CoIncrementMTAUsage;
use windows::Win32::System::Recovery::{
    RESTART_NO_CRASH, RESTART_NO_HANG, RESTART_NO_REBOOT, RegisterApplicationRestart,
};
use windows::Win32::UI::Shell::IInitializeWithWindow;
use windows::core::{HSTRING, Interface, Ref};
use windows_future::AsyncOperationProgressHandler;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Queue {
    Moving,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Taken {
    Done,
    Nothing,
    Canceled,
    Other(String),
}

type Numbered = (u16, u16, u16);

pub fn sold_here() -> bool {
    Package::Current()
        .and_then(|one| one.SignatureKind())
        .is_ok_and(|kind| kind == PackageSignatureKind::Store)
}

pub fn waiting_beside_us() -> Option<String> {
    let ours = Package::Current().ok()?;
    let family = ours.Id().ok()?.FamilyName().ok()?;
    let mine = released_part_of(&ours.Id().ok()?.Version().ok()?);
    let this_user = HSTRING::new();
    PackageManager::new()
        .ok()?
        .FindPackagesByUserSecurityIdPackageFamilyName(&this_user, &family)
        .ok()?
        .into_iter()
        .filter_map(|one| Some(released_part_of(&one.Id().ok()?.Version().ok()?)))
        .filter(|said| *said > mine)
        .max()
        .map(said)
}

pub fn offers(window: isize) -> windows::core::Result<bool> {
    let updates = shop(window)?
        .GetAppAndOptionalStorePackageUpdatesAsync()?
        .join()?;
    let ours = Package::Current()?.Id()?.FamilyName()?;
    for one in 0..updates.Size()? {
        if updates.GetAt(one)?.Package()?.Id()?.FamilyName()? == ours {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn queued(window: isize) -> windows::core::Result<Option<Queue>> {
    let ours = Package::Current()?.Id()?.FamilyName()?;
    let items = shop(window)?.GetAssociatedStoreQueueItemsAsync()?.join()?;
    for one in 0..items.Size()? {
        let item = items.GetAt(one)?;
        if item.PackageFamilyName().unwrap_or_default() != ours {
            continue;
        }
        let status = item.GetCurrentStatus()?;
        let state = status.PackageInstallState()?;
        if matches!(
            state,
            StoreQueueItemState::Active | StoreQueueItemState::Paused
        ) {
            let reason = status.PackageInstallExtendedState().unwrap_or_default();
            return Ok(Some(queue_of(state, reason)));
        }
    }
    Ok(None)
}

pub fn queue_of(state: StoreQueueItemState, reason: StoreQueueItemExtendedState) -> Queue {
    let waits_for_us_to_close = matches!(
        reason,
        StoreQueueItemExtendedState::PausedPackagesInUse
            | StoreQueueItemExtendedState::PausedReadyToInstall
    );
    if state == StoreQueueItemState::Paused && !waits_for_us_to_close {
        Queue::Paused
    } else {
        Queue::Moving
    }
}

pub fn take(window: isize, told: impl FnMut(f64) + Send + 'static) -> windows::core::Result<Taken> {
    let shop = shop(window)?;
    let updates = shop.GetAppAndOptionalStorePackageUpdatesAsync()?.join()?;
    if updates.Size()? == 0 {
        return Ok(Taken::Nothing);
    }
    let asking = shop.RequestDownloadAndInstallStorePackageUpdatesAsync(&updates)?;
    let told = std::sync::Mutex::new(told);
    asking.SetProgress(&AsyncOperationProgressHandler::new(
        move |_, far: Ref<'_, StorePackageUpdateStatus>| {
            let far = far.ok()?.PackageDownloadProgress;
            if let Ok(mut told) = told.lock() {
                told(far);
            }
            Ok(())
        },
    ))?;
    let done = asking.join()?;
    Ok(match done.OverallState()? {
        StorePackageUpdateState::Completed => Taken::Done,
        StorePackageUpdateState::Canceled => Taken::Canceled,
        other => Taken::Other(format!("{other:?}")),
    })
}

pub fn reopen_when_replaced() -> windows::core::Result<()> {
    let same_line = HSTRING::new();
    unsafe {
        RegisterApplicationRestart(
            &same_line,
            RESTART_NO_CRASH | RESTART_NO_HANG | RESTART_NO_REBOOT,
        )
    }
}

fn shop(window: isize) -> windows::core::Result<StoreContext> {
    join_the_apartment();
    let shop = StoreContext::GetDefault()?;
    let owner: IInitializeWithWindow = shop.cast()?;
    unsafe { owner.Initialize(HWND(window as *mut core::ffi::c_void)) }?;
    Ok(shop)
}

fn join_the_apartment() {
    static ONCE: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        let _ = unsafe { CoIncrementMTAUsage() };
    });
}

fn released_part_of(version: &PackageVersion) -> Numbered {
    (version.Major, version.Minor, version.Build)
}

pub fn said((major, minor, build): Numbered) -> String {
    format!("{major}.{minor}.{build}")
}

#[cfg(test)]
#[path = "store_test.rs"]
mod tests;
