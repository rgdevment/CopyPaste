fn what_goes_back_to_the_clipboard(b: &mut Battery) {
    b.group("I · Restore");

    b.case(
        "I1",
        "an item comes back to the clipboard with its formats",
        || {
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i1-original"))]);
            }
            let item = {
                let clipboard = Clipboard::open().ok_or("did not open")?;
                match capture(&clipboard) {
                    Captured::Kept(item) => item,
                    other => return Err(format!("nothing was captured: {other:?}")),
                }
            };
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i1-something-else"))]);
            }
            let written = {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                to_clipboard(&clipboard, &item)
            };
            match written {
                Restored::Written { formats, .. } => {
                    println!("            {formats} formats returned");
                }
                other => return Err(format!("not restored: {other:?}")),
            }
            let clipboard = Clipboard::open().ok_or("did not open")?;
            let bytes = clipboard.bytes(CF_UNICODETEXT).ok_or("no text")?;
            match text_of(&bytes).as_deref() {
                Some("cp-i1-original") => Ok(()),
                other => Err(format!("came back «{other:?}»")),
            }
        },
    );

    b.case(
        "I2",
        "capturing what was restored gives the same fingerprint",
        || {
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i2-round-trip"))]);
            }
            let (first, item) = {
                let clipboard = Clipboard::open().ok_or("did not open")?;
                match capture(&clipboard) {
                    Captured::Kept(item) => (item.fingerprint(), item),
                    other => return Err(format!("nothing was captured: {other:?}")),
                }
            };
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                to_clipboard(&clipboard, &item);
            }
            let clipboard = Clipboard::open().ok_or("did not open")?;
            match capture(&clipboard) {
                Captured::Kept(again) => {
                    if again.fingerprint() == first {
                        Ok(())
                    } else {
                        Err("the fingerprint changed on the round trip".into())
                    }
                }
                other => Err(format!("not recaptured: {other:?}")),
            }
        },
    );

    b.case(
        "I3",
        "pasting as plain text does not mutilate the stored item",
        || {
            let html = id_of("HTML Format").ok_or("HTML Format did not register")?;
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[
                    (CF_UNICODETEXT, &utf16_of("with styles")),
                    (html, b"<b>with styles</b>"),
                ]);
            }
            let item = {
                let clipboard = Clipboard::open().ok_or("did not open")?;
                capture(&clipboard).kept().ok_or("nothing was captured")?
            };
            let had = item.formats.len();

            let plain = render(Form::PlainText, &content_of(&item, None))
                .ok_or("the plain form was not offered")?
                .into_item();
            let written = {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                to_clipboard(&clipboard, &plain)
            };
            match written {
                Restored::Written { formats: 1, .. } => {}
                other => return Err(format!("returned {other:?}")),
            }
            {
                let clipboard = Clipboard::open().ok_or("did not open")?;
                if clipboard.offered().contains(&html) {
                    return Err("the HTML remained: it was not pasted as plain text".into());
                }
            }

            if item.formats.len() != had {
                return Err("the item lost formats".into());
            }
            let restored = {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                to_clipboard(&clipboard, &item)
            };
            match restored {
                Restored::Written { .. } => {}
                other => return Err(format!("could not restore with styles: {other:?}")),
            }
            let clipboard = Clipboard::open().ok_or("did not open")?;
            if !clipboard.offered().contains(&html) {
                return Err("the HTML did not come back: the item had been left mutilated".into());
            }
            Ok(())
        },
    );

    b.case(
        "I4",
        "an item with no plain text cannot be pasted as plain text",
        || {
            let only_image = cp_core::item::Item {
                kind: Some(cp_core::kind::Kind::Image),
                formats: vec![cp_core::item::Format {
                    id: "PNG".into(),
                    payload: cp_core::item::Payload::Inline(vec![1, 2, 3]),
                }],
            };
            let content = content_of(&only_image, None);
            if forms_for(&content).contains(&Form::PlainText) {
                return Err("pasting as plain text was offered without text".into());
            }
            match render(Form::PlainText, &content) {
                None => Ok(()),
                Some(other) => Err(format!("returned {other:?}")),
            }
        },
    );

    b.case(
        "I5",
        "an image from the history comes back as a bitmap, not only as a private PNG",
        || {
            let png =
                std::fs::read("fixtures/texto-en-imagen.png").map_err(|why| why.to_string())?;
            let stored = cp_core::item::Item {
                kind: Some(cp_core::kind::Kind::Image),
                formats: vec![
                    cp_core::item::Format {
                        id: "CF_BITMAP".into(),
                        payload: cp_core::item::Payload::Announced { size: None },
                    },
                    cp_core::item::Format {
                        id: "CF_DIB".into(),
                        payload: cp_core::item::Payload::Announced {
                            size: Some(585_228),
                        },
                    },
                    cp_core::item::Format {
                        id: "CF_DIBV5".into(),
                        payload: cp_core::item::Payload::Announced {
                            size: Some(585_312),
                        },
                    },
                    cp_core::item::Format {
                        id: "PNG".into(),
                        payload: cp_core::item::Payload::Inline(png.clone()),
                    },
                ],
            };
            {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                clipboard.replace(&[(CF_UNICODETEXT, &utf16_of("cp-i5-not-an-image"))]);
            }
            let written = {
                let clipboard = Clipboard::to_write().ok_or("did not open")?;
                to_clipboard(&clipboard, &stored)
            };
            match written {
                Restored::Written { formats, .. } => {
                    println!("            {formats} formats returned");
                }
                other => return Err(format!("not restored: {other:?}")),
            }
            let clipboard = Clipboard::open().ok_or("did not open")?;
            let modern = clipboard.bytes(CF_DIBV5).map(|bytes| bytes.len());
            let classic = clipboard.bytes(CF_DIB).map(|bytes| bytes.len());
            println!("            CF_DIBV5 {modern:?}, CF_DIB {classic:?}");
            if modern.is_none() && classic.is_none() {
                return Err(
                    "only the private PNG travelled: anything that wants a bitmap pastes nothing"
                        .into(),
                );
            }
            match clipboard.bytes(id_of("PNG").ok_or("the PNG name is not registered")?) {
                Some(bytes) if bytes == png => Ok(()),
                Some(bytes) => Err(format!("the PNG came back with {} bytes", bytes.len())),
                None => Err("the PNG did not travel".into()),
            }
        },
    );
}
