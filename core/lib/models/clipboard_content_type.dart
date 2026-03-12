enum ClipboardContentType {
  unknown,
  text,
  image,
  file,
  folder,
  link,
  audio,
  video;

  static ClipboardContentType fromValue(int value) => switch (value) {
    0 => text,
    1 => image,
    2 => file,
    3 => folder,
    4 => link,
    5 => audio,
    6 => video,
    _ => unknown,
  };

  int get value => switch (this) {
    unknown => -1,
    text => 0,
    image => 1,
    file => 2,
    folder => 3,
    link => 4,
    audio => 5,
    video => 6,
  };
}
