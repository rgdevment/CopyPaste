enum CardColor {
  none(0, 0x00000000),
  red(1, 0xFFE74C3C),
  green(2, 0xFF2ECC71),
  purple(3, 0xFF9B59B6),
  yellow(4, 0xFFF1C40F),
  blue(5, 0xFF3498DB),
  orange(6, 0xFFE67E22);

  const CardColor(this.value, this.argb);

  final int value;
  final int argb;

  static CardColor fromValue(int value) =>
      CardColor.values.firstWhere((c) => c.value == value, orElse: () => none);
}
