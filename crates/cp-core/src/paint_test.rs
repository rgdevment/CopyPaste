use super::*;

fn opaque(red: u8, green: u8, blue: u8) -> Option<Rgba> {
    Some(Rgba {
        red,
        green,
        blue,
        alpha: 0xFF,
    })
}

#[test]
fn six_digits_are_read_a_pair_at_a_time() {
    assert_eq!(rgba_of("#FF8800"), opaque(0xFF, 0x88, 0x00));
    assert_eq!(rgba_of("#ff8800"), opaque(0xFF, 0x88, 0x00));
    assert_eq!(rgba_of("  #000000  "), opaque(0, 0, 0));
    assert_eq!(rgba_of("#FFFFFF"), opaque(255, 255, 255));
}

#[test]
fn three_digits_mean_each_one_twice() {
    assert_eq!(rgba_of("#F80"), rgba_of("#FF8800"));
    assert_eq!(rgba_of("#000"), opaque(0, 0, 0));
    assert_eq!(rgba_of("#fff"), opaque(255, 255, 255));
}

#[test]
fn eight_digits_carry_how_see_through_it_is() {
    assert_eq!(
        rgba_of("#FF880080"),
        Some(Rgba {
            red: 0xFF,
            green: 0x88,
            blue: 0x00,
            alpha: 0x80
        })
    );
    assert_eq!(rgba_of("#FF8800FF"), rgba_of("#FF8800"));
}

#[test]
fn what_is_not_a_colour_is_not_guessed_at() {
    for said in [
        "",
        "#",
        "#G",
        "#GGG",
        "#12",
        "#12345",
        "#1234567",
        "#123456789",
        "copypaste",
        "rgb(1, 2)",
        "rgb(1, 2, 3, 4, 5)",
        "hsl()",
        "rgb(a, b, c)",
    ] {
        assert_eq!(rgba_of(said), None, "«{said}»");
    }
}

#[test]
fn the_three_numbers_of_rgb_are_channels_as_they_are() {
    assert_eq!(rgba_of("rgb(255, 136, 0)"), opaque(0xFF, 0x88, 0x00));
    assert_eq!(rgba_of("RGB(255,136,0)"), opaque(0xFF, 0x88, 0x00));
    assert_eq!(rgba_of("rgb(0,0,0)"), opaque(0, 0, 0));
}

#[test]
fn a_fourth_number_is_how_opaque_and_percentages_are_shares() {
    assert_eq!(
        rgba_of("rgba(255, 136, 0, 0.5)"),
        Some(Rgba {
            red: 0xFF,
            green: 0x88,
            blue: 0x00,
            alpha: 128
        })
    );
    assert_eq!(rgba_of("rgba(255, 136, 0, 1)"), rgba_of("#FF8800"));
    assert_eq!(rgba_of("rgb(100%, 0%, 0%)"), opaque(255, 0, 0));
}

#[test]
fn a_channel_out_of_range_is_brought_back_in() {
    assert_eq!(rgba_of("rgb(300, -20, 0)"), opaque(255, 0, 0));
    assert_eq!(
        rgba_of("rgba(0, 0, 0, 9)"),
        Some(Rgba {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 255
        })
    );
}

#[test]
fn the_wheel_of_hsl_lands_on_the_colours_it_should() {
    assert_eq!(rgba_of("hsl(0, 100%, 50%)"), opaque(255, 0, 0));
    assert_eq!(rgba_of("hsl(120, 100%, 50%)"), opaque(0, 255, 0));
    assert_eq!(rgba_of("hsl(240, 100%, 50%)"), opaque(0, 0, 255));
    assert_eq!(rgba_of("hsl(60, 100%, 50%)"), opaque(255, 255, 0));
    assert_eq!(rgba_of("hsl(180, 100%, 50%)"), opaque(0, 255, 255));
    assert_eq!(rgba_of("hsl(300, 100%, 50%)"), opaque(255, 0, 255));
}

#[test]
fn no_colour_at_all_is_grey_whatever_the_angle() {
    for turn in [0, 90, 200, 359] {
        assert_eq!(
            rgba_of(&format!("hsl({turn}, 0%, 50%)")),
            opaque(128, 128, 128),
            "{turn} degrees"
        );
    }
    assert_eq!(rgba_of("hsl(0, 100%, 0%)"), opaque(0, 0, 0));
    assert_eq!(rgba_of("hsl(0, 100%, 100%)"), opaque(255, 255, 255));
}

#[test]
fn the_angle_goes_round_and_round() {
    assert_eq!(rgba_of("hsl(360, 100%, 50%)"), rgba_of("hsl(0, 100%, 50%)"));
    assert_eq!(
        rgba_of("hsl(480, 100%, 50%)"),
        rgba_of("hsl(120, 100%, 50%)")
    );
    assert_eq!(
        rgba_of("hsl(-120, 100%, 50%)"),
        rgba_of("hsl(240, 100%, 50%)")
    );
    assert_eq!(
        rgba_of("hsl(0deg, 100%, 50%)"),
        rgba_of("hsl(0, 100%, 50%)")
    );
}

#[test]
fn hsla_carries_how_see_through_it_is_too() {
    assert_eq!(
        rgba_of("hsla(0, 100%, 50%, 0.5)"),
        Some(Rgba {
            red: 255,
            green: 0,
            blue: 0,
            alpha: 128
        })
    );
}

#[test]
fn whatever_the_classifier_calls_a_colour_can_be_shown_as_one() {
    for said in [
        "#FF8800",
        "#F80",
        "#FF880080",
        "rgb(1, 2, 3)",
        "rgba(1, 2, 3, 0.4)",
        "hsl(1, 2%, 3%)",
        "hsla(1, 2%, 3%, 0.4)",
    ] {
        assert!(
            rgba_of(said).is_some(),
            "«{said}» is filed as a colour, so the card has to be able to draw it"
        );
    }
}
