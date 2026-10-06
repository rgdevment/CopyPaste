fn the_panel_and_its_windows(b: &mut Battery) {
    b.group("S · The panel and the settings window");

    let floated = cp_mac_sys::floating::a_panel_would_float();

    b.case(
        "S1",
        "the panel becomes a panel that takes the keys without waking the app",
        || {
            let floated = floated.ok_or("this needs the main thread and a window of its own")?;
            if !floated.became_a_panel {
                return Err("the window never became a panel".into());
            }
            if !floated.never_activates {
                return Err("the panel would bring the whole app forward".into());
            }
            Ok(())
        },
    );

    b.case(
        "S2",
        "the panel shows up over an app in full screen and on the space in use",
        || {
            let floated = floated.ok_or("this needs the main thread and a window of its own")?;
            if !floated.joins_full_screen {
                return Err("over a full screen app the panel would open on another space".into());
            }
            if !floated.follows_the_space {
                return Err("the panel would drag the user back to where it was last shown".into());
            }
            Ok(())
        },
    );

    b.case(
        "S3",
        "the panel goes back to the window it was, so closing it with an observer on it does not abort",
        || {
            let floated = floated.ok_or("this needs the main thread and a window of its own")?;
            if !floated.back_as_it_was {
                return Err("the window kept the panel class, and AppKit aborts when it is let go".into());
            }
            Ok(())
        },
    );

    b.case_or_skip("S4", "the panel takes the keys when it is put in front", || {
        let floated = floated.ok_or("this needs the main thread and a window of its own")?;
        if floated.took_the_keys {
            if !floated.counts_as_ours {
                return Err("the panel holds the keys yet the app does not count it as its own".into());
            }
            return Ok(());
        }
        Err(format!(
            "{SKIPPED}no window can take the keys without a session on screen"
        ))
    });

    b.case(
        "S5",
        "the settings window keeps close and minimise and loses zoom",
        || {
            let buttons = cp_mac_sys::titlebar::a_titled_window_would_lose_zoom()
                .ok_or("this needs the main thread and a window with buttons")?;
            if buttons.zoom_shown {
                return Err("the green button is still there".into());
            }
            if !buttons.close_shown || !buttons.minimise_shown {
                return Err(format!("the other buttons went with it: {buttons:?}"));
            }
            Ok(())
        },
    );

    b.case(
        "S6",
        "the pointer sits on a screen whose visible part holds it",
        || {
            let spot = cp_mac_sys::pointer::spot().ok_or("this needs the main thread and a screen")?;
            if spot.right <= spot.left || spot.bottom <= spot.top {
                return Err(format!("the visible part of the screen is empty: {spot:?}"));
            }
            Ok(())
        },
    );

    b.case(
        "S7",
        "the panel stays put when the app loses focus and shows without an animation of its own",
        || {
            let floated = floated.ok_or("this needs the main thread and a window of its own")?;
            if !floated.stays_when_left {
                return Err("AppKit would hide the panel behind the app's back".into());
            }
            if !floated.appears_at_once {
                return Err("AppKit would animate the panel in and out".into());
            }
            Ok(())
        },
    );
}
