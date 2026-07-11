glyphr::generate_font! {
    name: B612_SMALL,
    path: "assets/B612-Regular.ttf",
    size: 18,
    characters: "0-9A-Za-z! /:,%",
    format: Bitmap {
        spread: 10.0,
        padding: 0
    }
}

glyphr::generate_font! {
    name: B612_REGULAR,
    path: "assets/B612-Regular.ttf",
    size: 24,
    characters: "0-9A-Za-z! /:,%",
    format: Bitmap {
        spread: 10.0,
        padding: 0
    }
}

glyphr::generate_font! {
    name: ICONS,
    path: "/System/Library/Fonts/Menlo.ttc",
    size: 24,
    characters: "☼☉!",
    format: Bitmap {
        spread: 10.0,
        padding: 0
    }
}
glyphr::generate_font! {
    name: B612_REGULAR_LARGE_NUMBERS,
    path: "assets/B612-Regular.ttf",
    size: 36,
    characters: "0-9.-",
    format: Bitmap {
        spread: 20.0,
        padding: 0
    }
}

glyphr::generate_font! {
    name: B612_REGULAR_VERY_LARGE_NUMBERS,
    path: "assets/B612-Regular.ttf",
    size: 48,
    characters: "0-9.-",
    format: Bitmap {
        spread: 20.0,
        padding: 0
    }
}
