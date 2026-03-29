import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  group('TextClassifier edge cases', () {
    group('multiline content always falls back to text or json', () {
      test('multiline that starts with { but not valid JSON is text', () {
        expect(
          TextClassifier.classify('{ key: value\nno quotes }'),
          equals(ClipboardContentType.text),
        );
      });

      test('multiline valid JSON object is json', () {
        expect(
          TextClassifier.classify('{\n  "a": 1,\n  "b": 2\n}'),
          equals(ClipboardContentType.json),
        );
      });

      test('multiline that looks like email but has newline is text', () {
        // newline prevents single-line email detection
        expect(
          TextClassifier.classify('user@example.com\nextra'),
          equals(ClipboardContentType.text),
        );
      });

      test('UUID with only trailing newline is still uuid after trim()', () {
        // classify() calls trim() first — trailing newline removed before
        // the contains('\n') guard, so the UUID pattern is still matched.
        expect(
          TextClassifier.classify('550e8400-e29b-41d4-a716-446655440000\n'),
          equals(ClipboardContentType.uuid),
        );
      });

      test('UUID embedded in multiline text is treated as text', () {
        // A real newline in the middle means contains('\n') is true
        expect(
          TextClassifier.classify(
            '550e8400-e29b-41d4-a716-446655440000\nmore text',
          ),
          equals(ClipboardContentType.text),
        );
      });
    });

    group('email edge cases', () {
      test('email with underscore in local part', () {
        expect(
          TextClassifier.classify('first_last@company.org'),
          equals(ClipboardContentType.email),
        );
      });

      test('email with dots in local part', () {
        expect(
          TextClassifier.classify('first.last@company.io'),
          equals(ClipboardContentType.email),
        );
      });

      test('email with hyphen in domain', () {
        expect(
          TextClassifier.classify('user@my-company.com'),
          equals(ClipboardContentType.email),
        );
      });

      test('text with @ but missing TLD is not email', () {
        expect(
          TextClassifier.classify('user@host'),
          isNot(equals(ClipboardContentType.email)),
        );
      });

      test('text with multiple @ signs is not email', () {
        expect(
          TextClassifier.classify('user@@domain.com'),
          isNot(equals(ClipboardContentType.email)),
        );
      });
    });

    group('color edge cases', () {
      test('RGB with no spaces is color', () {
        expect(
          TextClassifier.classify('rgb(255,87,51)'),
          equals(ClipboardContentType.color),
        );
      });

      test('HSL uppercase is color', () {
        expect(
          TextClassifier.classify('HSL(14, 100%, 51%)'),
          equals(ClipboardContentType.color),
        );
      });

      test('7-digit hex is not a color', () {
        expect(
          TextClassifier.classify('#FF573'),
          isNot(equals(ClipboardContentType.color)),
        );
      });

      test('hex without hash prefix is not color', () {
        expect(
          TextClassifier.classify('FF5733'),
          isNot(equals(ClipboardContentType.color)),
        );
      });

      test('RGBA with decimal alpha is color', () {
        expect(
          TextClassifier.classify('rgba(10, 20, 30, 0.9)'),
          equals(ClipboardContentType.color),
        );
      });
    });

    group('IP address edge cases', () {
      test('leading zeros in octet — edge of valid range', () {
        // 010.0.0.1 — parser accepts 01 as valid per regex
        final result = TextClassifier.classify('010.0.0.1');
        // The regex accepts [01]?\d\d?, so 010 is matched as valid
        expect(result, equals(ClipboardContentType.ip));
      });

      test('single-octet number is not an IP', () {
        expect(
          TextClassifier.classify('192'),
          isNot(equals(ClipboardContentType.ip)),
        );
      });

      test('IP with trailing dot is not valid', () {
        expect(
          TextClassifier.classify('192.168.1.1.'),
          isNot(equals(ClipboardContentType.ip)),
        );
      });

      test('loopback 127.0.0.1 is IP', () {
        expect(
          TextClassifier.classify('127.0.0.1'),
          equals(ClipboardContentType.ip),
        );
      });
    });

    group('UUID edge cases', () {
      test('UUID with mixed case is uuid', () {
        expect(
          TextClassifier.classify('550E8400-e29b-41D4-A716-446655440000'),
          equals(ClipboardContentType.uuid),
        );
      });

      test('UUID with wrong segment lengths is not uuid', () {
        expect(
          TextClassifier.classify('550e8400-e29b-41d4-a716-44665544000'),
          isNot(equals(ClipboardContentType.uuid)),
        );
      });

      test('UUID with extra characters is not uuid', () {
        expect(
          TextClassifier.classify('550e8400-e29b-41d4-a716-4466554400001'),
          isNot(equals(ClipboardContentType.uuid)),
        );
      });
    });

    group('JSON edge cases', () {
      test('empty JSON object is json', () {
        expect(
          TextClassifier.classify('{}'),
          equals(ClipboardContentType.json),
        );
      });

      test('empty JSON array is json', () {
        expect(
          TextClassifier.classify('[]'),
          equals(ClipboardContentType.json),
        );
      });

      test('JSON with only numbers is json', () {
        expect(
          TextClassifier.classify('[1, 2, 3, 4]'),
          equals(ClipboardContentType.json),
        );
      });

      test('JSON with boolean values is json', () {
        expect(
          TextClassifier.classify('{"active": true, "count": 0}'),
          equals(ClipboardContentType.json),
        );
      });

      test('bare number is not json', () {
        expect(
          TextClassifier.classify('42'),
          isNot(equals(ClipboardContentType.json)),
        );
      });

      test('bare string is not json', () {
        expect(
          TextClassifier.classify('"hello"'),
          isNot(equals(ClipboardContentType.json)),
        );
      });

      test('JSON starting with [ but invalid is not json', () {
        expect(
          TextClassifier.classify('[unclosed'),
          isNot(equals(ClipboardContentType.json)),
        );
      });
    });

    group('phone edge cases', () {
      test('too-short phone is not phone', () {
        expect(
          TextClassifier.classify('+1 234'),
          isNot(equals(ClipboardContentType.phone)),
        );
      });

      test('phone at exactly 7 digits (minimum) is phone', () {
        expect(
          TextClassifier.classify('+1 234 567'),
          equals(ClipboardContentType.phone),
        );
      });

      test('phone with 14 digits (within 7–15 range) is phone', () {
        // +1 234 567 890 1234 → digits: 12345678901234 = 14 digits, within range
        expect(
          TextClassifier.classify('+1 234 567 890 1234'),
          equals(ClipboardContentType.phone),
        );
      });

      test('phone exceeding 15 digits (E.164 max) is not phone', () {
        // 16 consecutive digits after + → _isPhone digit count check rejects it
        // (regex matches because no spaces, but digit count > 15 fails)
        expect(
          TextClassifier.classify('+1234567890123456'),
          isNot(equals(ClipboardContentType.phone)),
        );
      });

      test('area-code format without + still works', () {
        expect(
          TextClassifier.classify('(55) 2222-2222'),
          equals(ClipboardContentType.phone),
        );
      });
    });

    group('whitespace handling', () {
      test('content with only tabs is text', () {
        expect(
          TextClassifier.classify('\t\t\t'),
          equals(ClipboardContentType.text),
        );
      });

      test('content with leading/trailing whitespace is trimmed', () {
        // Email with surrounding spaces should still be detected
        expect(
          TextClassifier.classify('  user@example.com  '),
          equals(ClipboardContentType.email),
        );
      });
    });
  });
}
