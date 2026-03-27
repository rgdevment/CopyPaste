import 'package:flutter_test/flutter_test.dart';

import 'package:core/core.dart';

void main() {
  group('TextClassifier.classify', () {
    group('email', () {
      test('detects standard email', () {
        expect(
          TextClassifier.classify('user@gmail.com'),
          ClipboardContentType.email,
        );
      });

      test('detects email with subdomains', () {
        expect(
          TextClassifier.classify('user@mail.company.co.uk'),
          ClipboardContentType.email,
        );
      });

      test('detects email with plus alias', () {
        expect(
          TextClassifier.classify('user+filter@proton.me'),
          ClipboardContentType.email,
        );
      });

      test('does not classify plain text as email', () {
        expect(
          TextClassifier.classify('not an email'),
          ClipboardContentType.text,
        );
      });

      test('does not classify incomplete email', () {
        expect(
          TextClassifier.classify('@gmail.com'),
          ClipboardContentType.text,
        );
      });
    });

    group('phone', () {
      test('detects international phone with +', () {
        expect(
          TextClassifier.classify('+56 9 1234 5678'),
          ClipboardContentType.phone,
        );
      });

      test('detects phone with dashes', () {
        expect(
          TextClassifier.classify('+1-800-555-0100'),
          ClipboardContentType.phone,
        );
      });

      test('detects phone with parentheses', () {
        expect(
          TextClassifier.classify('+44 (20) 7946 0958'),
          ClipboardContentType.phone,
        );
      });

      test('does not classify short digit sequence as phone', () {
        expect(
          TextClassifier.classify('12345'),
          isNot(ClipboardContentType.phone),
        );
      });

      test('does not classify 16+ digit string as phone', () {
        expect(
          TextClassifier.classify('+1 800 555 0100 12345'),
          isNot(ClipboardContentType.phone),
        );
      });
    });

    group('color', () {
      test('detects 6-digit hex', () {
        expect(TextClassifier.classify('#FF5733'), ClipboardContentType.color);
      });

      test('detects 3-digit hex', () {
        expect(TextClassifier.classify('#F57'), ClipboardContentType.color);
      });

      test('detects 8-digit hex with alpha', () {
        expect(
          TextClassifier.classify('#FF5733AA'),
          ClipboardContentType.color,
        );
      });

      test('detects rgb()', () {
        expect(
          TextClassifier.classify('rgb(255, 87, 51)'),
          ClipboardContentType.color,
        );
      });

      test('detects rgba()', () {
        expect(
          TextClassifier.classify('rgba(255, 87, 51, 0.5)'),
          ClipboardContentType.color,
        );
      });

      test('detects hsl()', () {
        expect(
          TextClassifier.classify('hsl(14, 100%, 51%)'),
          ClipboardContentType.color,
        );
      });

      test('detects hsla()', () {
        expect(
          TextClassifier.classify('hsla(14, 100%, 51%, 0.8)'),
          ClipboardContentType.color,
        );
      });

      test('does not classify arbitrary hash as color', () {
        expect(
          TextClassifier.classify('#ZZZZZZ'),
          isNot(ClipboardContentType.color),
        );
      });
    });

    group('ip address', () {
      test('detects valid IPv4', () {
        expect(TextClassifier.classify('192.168.1.1'), ClipboardContentType.ip);
      });

      test('detects edge case 0.0.0.0', () {
        expect(TextClassifier.classify('0.0.0.0'), ClipboardContentType.ip);
      });

      test('detects 255.255.255.255', () {
        expect(
          TextClassifier.classify('255.255.255.255'),
          ClipboardContentType.ip,
        );
      });

      test('does not classify out-of-range octet', () {
        expect(
          TextClassifier.classify('256.0.0.1'),
          isNot(ClipboardContentType.ip),
        );
      });

      test('does not classify partial IP', () {
        expect(
          TextClassifier.classify('192.168.1'),
          isNot(ClipboardContentType.ip),
        );
      });
    });

    group('uuid', () {
      test('detects v4 UUID', () {
        expect(
          TextClassifier.classify('550e8400-e29b-41d4-a716-446655440000'),
          ClipboardContentType.uuid,
        );
      });

      test('detects uppercase UUID', () {
        expect(
          TextClassifier.classify('550E8400-E29B-41D4-A716-446655440000'),
          ClipboardContentType.uuid,
        );
      });

      test('does not classify malformed UUID', () {
        expect(
          TextClassifier.classify('550e8400-e29b-41d4-a716'),
          isNot(ClipboardContentType.uuid),
        );
      });
    });

    group('json', () {
      test('detects JSON object', () {
        expect(
          TextClassifier.classify('{"key": "value"}'),
          ClipboardContentType.json,
        );
      });

      test('detects JSON array', () {
        expect(TextClassifier.classify('[1, 2, 3]'), ClipboardContentType.json);
      });

      test('detects multiline JSON', () {
        expect(
          TextClassifier.classify('{\n  "name": "Mario",\n  "age": 30\n}'),
          ClipboardContentType.json,
        );
      });

      test('detects nested JSON', () {
        expect(
          TextClassifier.classify('{"user": {"id": 1, "tags": ["a", "b"]}}'),
          ClipboardContentType.json,
        );
      });

      test('does not classify invalid JSON', () {
        expect(
          TextClassifier.classify('{invalid json}'),
          isNot(ClipboardContentType.json),
        );
      });

      test('does not classify plain object-like text as json', () {
        expect(
          TextClassifier.classify('{not json at all'),
          isNot(ClipboardContentType.json),
        );
      });
    });

    group('text fallback', () {
      test('classifies empty string as text', () {
        expect(TextClassifier.classify(''), ClipboardContentType.text);
      });

      test('classifies plain sentence as text', () {
        expect(
          TextClassifier.classify('Hello world'),
          ClipboardContentType.text,
        );
      });

      test('classifies multiline prose as text', () {
        expect(
          TextClassifier.classify('Line one\nLine two\nLine three'),
          ClipboardContentType.text,
        );
      });

      test('classifies whitespace-only as text', () {
        expect(TextClassifier.classify('   '), ClipboardContentType.text);
      });
    });

    group('priority ordering', () {
      test('email takes priority over phone-like pattern', () {
        expect(
          TextClassifier.classify('user@example.com'),
          ClipboardContentType.email,
        );
      });

      test('uuid not confused with plain hex string', () {
        expect(
          TextClassifier.classify('550e8400e29b41d4a716446655440000'),
          isNot(ClipboardContentType.uuid),
        );
      });
    });
  });
}
