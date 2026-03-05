class SearchHelper {
  SearchHelper._();

  static final _combiningDiacritics = RegExp(r'[\u0300-\u036f]');

  static String normalize(String text) {
    // Unicode NFD decomposition splits base chars from combining marks,
    // then we strip the combining diacritical marks (U+0300–U+036F).
    // Finally, handle special ligatures that NFD doesn't decompose.
    var result = text.toLowerCase();

    // NFD decomposes e.g. 'é' → 'e' + '\u0301'
    // Dart strings are UTF-16; we can approximate NFD by using
    // the runes-based approach with the combining marks regex.
    // Dart doesn't have a built-in normalize(), so we use a manual
    // decomposition for the most common cases plus strip combining marks.
    result = _expandLigatures(result);
    result = _decomposeToNfd(result);
    result = result.replaceAll(_combiningDiacritics, '');

    return result;
  }

  static String _expandLigatures(String text) {
    return text
        .replaceAll('ß', 'ss')
        .replaceAll('æ', 'ae')
        .replaceAll('œ', 'oe')
        .replaceAll('ð', 'd')
        .replaceAll('þ', 'th')
        .replaceAll('ł', 'l')
        .replaceAll('đ', 'd');
  }

  static String _decomposeToNfd(String text) {
    final buffer = StringBuffer();
    for (final rune in text.runes) {
      final decomposed = _nfdMap[rune];
      if (decomposed != null) {
        buffer.write(decomposed);
      } else {
        buffer.writeCharCode(rune);
      }
    }
    return buffer.toString();
  }

  // NFD decomposition map for common accented characters.
  // Maps composed Unicode codepoint → base char + combining mark(s).
  // The combining marks are then stripped by _combiningDiacritics regex.
  static final Map<int, String> _nfdMap = {
    // Latin lowercase
    0x00E0: 'a\u0300', // à
    0x00E1: 'a\u0301', // á
    0x00E2: 'a\u0302', // â
    0x00E3: 'a\u0303', // ã
    0x00E4: 'a\u0308', // ä
    0x00E5: 'a\u030A', // å
    0x00E7: 'c\u0327', // ç
    0x00E8: 'e\u0300', // è
    0x00E9: 'e\u0301', // é
    0x00EA: 'e\u0302', // ê
    0x00EB: 'e\u0308', // ë
    0x00EC: 'i\u0300', // ì
    0x00ED: 'i\u0301', // í
    0x00EE: 'i\u0302', // î
    0x00EF: 'i\u0308', // ï
    0x00F1: 'n\u0303', // ñ
    0x00F2: 'o\u0300', // ò
    0x00F3: 'o\u0301', // ó
    0x00F4: 'o\u0302', // ô
    0x00F5: 'o\u0303', // õ
    0x00F6: 'o\u0308', // ö
    0x00F8: 'o',       // ø (no combining mark, just strip)
    0x00F9: 'u\u0300', // ù
    0x00FA: 'u\u0301', // ú
    0x00FB: 'u\u0302', // û
    0x00FC: 'u\u0308', // ü
    0x00FD: 'y\u0301', // ý
    0x00FF: 'y\u0308', // ÿ
    // Latin uppercase (also lowercased before reaching here, but safety net)
    0x00C0: 'a\u0300', // À
    0x00C1: 'a\u0301', // Á
    0x00C2: 'a\u0302', // Â
    0x00C3: 'a\u0303', // Ã
    0x00C4: 'a\u0308', // Ä
    0x00C5: 'a\u030A', // Å
    0x00C7: 'c\u0327', // Ç
    0x00C8: 'e\u0300', // È
    0x00C9: 'e\u0301', // É
    0x00CA: 'e\u0302', // Ê
    0x00CB: 'e\u0308', // Ë
    0x00CC: 'i\u0300', // Ì
    0x00CD: 'i\u0301', // Í
    0x00CE: 'i\u0302', // Î
    0x00CF: 'i\u0308', // Ï
    0x00D1: 'n\u0303', // Ñ
    0x00D2: 'o\u0300', // Ò
    0x00D3: 'o\u0301', // Ó
    0x00D4: 'o\u0302', // Ô
    0x00D5: 'o\u0303', // Õ
    0x00D6: 'o\u0308', // Ö
    0x00D8: 'o',       // Ø
    0x00D9: 'u\u0300', // Ù
    0x00DA: 'u\u0301', // Ú
    0x00DB: 'u\u0302', // Û
    0x00DC: 'u\u0308', // Ü
    0x00DD: 'y\u0301', // Ý
    // Extended Latin
    0x0100: 'a',       // Ā
    0x0101: 'a',       // ā
    0x0102: 'a',       // Ă
    0x0103: 'a',       // ă
    0x0104: 'a',       // Ą
    0x0105: 'a',       // ą
    0x0106: 'c',       // Ć
    0x0107: 'c',       // ć
    0x010C: 'c',       // Č
    0x010D: 'c',       // č
    0x010E: 'd',       // Ď
    0x010F: 'd',       // ď
    0x0112: 'e',       // Ē
    0x0113: 'e',       // ē
    0x0116: 'e',       // Ė
    0x0117: 'e',       // ė
    0x0118: 'e',       // Ę
    0x0119: 'e',       // ę
    0x011A: 'e',       // Ě
    0x011B: 'e',       // ě
    0x011E: 'g',       // Ğ
    0x011F: 'g',       // ğ
    0x0130: 'i',       // İ
    0x0131: 'i',       // ı
    0x0141: 'l',       // Ł (handled in ligatures too)
    0x0142: 'l',       // ł
    0x0143: 'n',       // Ń
    0x0144: 'n',       // ń
    0x0147: 'n',       // Ň
    0x0148: 'n',       // ň
    0x0150: 'o',       // Ő
    0x0151: 'o',       // ő
    0x0154: 'r',       // Ŕ
    0x0155: 'r',       // ŕ
    0x0158: 'r',       // Ř
    0x0159: 'r',       // ř
    0x015A: 's',       // Ś
    0x015B: 's',       // ś
    0x015E: 's',       // Ş
    0x015F: 's',       // ş
    0x0160: 's',       // Š
    0x0161: 's',       // š
    0x0162: 't',       // Ţ
    0x0163: 't',       // ţ
    0x0164: 't',       // Ť
    0x0165: 't',       // ť
    0x016E: 'u',       // Ů
    0x016F: 'u',       // ů
    0x0170: 'u',       // Ű
    0x0171: 'u',       // ű
    0x017D: 'z',       // Ž
    0x017E: 'z',       // ž
    0x0179: 'z',       // Ź
    0x017A: 'z',       // ź
    0x017B: 'z',       // Ż
    0x017C: 'z',       // ż
  };
}
