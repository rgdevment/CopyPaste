import 'dart:convert';

import 'package:copypaste/services/manifest_signature.dart';
import 'package:cryptography/cryptography.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('ManifestSignature', () {
    late SimpleKeyPair keyPair;
    late SimplePublicKey publicKey;
    late List<int> publicKeyBytes;

    setUp(() async {
      keyPair = await Ed25519().newKeyPair();
      publicKey = await keyPair.extractPublicKey();
      publicKeyBytes = publicKey.bytes;
      ManifestSignature.reset();
      ManifestSignature.overridePublicKey(publicKeyBytes);
    });

    tearDown(ManifestSignature.reset);

    test('verifies a valid signature', () async {
      final body = utf8.encode('{"hello":"world"}');
      final sig = await Ed25519().sign(body, keyPair: keyPair);
      final ok = await ManifestSignature.verify(body, base64Encode(sig.bytes));
      expect(ok, isTrue);
    });

    test('rejects a tampered payload', () async {
      final body = utf8.encode('{"hello":"world"}');
      final sig = await Ed25519().sign(body, keyPair: keyPair);
      final tampered = utf8.encode('{"hello":"WORLD"}');
      final ok = await ManifestSignature.verify(
        tampered,
        base64Encode(sig.bytes),
      );
      expect(ok, isFalse);
    });

    test('rejects a malformed signature string', () async {
      final body = utf8.encode('payload');
      final ok = await ManifestSignature.verify(body, '!!!not-base64!!!');
      expect(ok, isFalse);
    });

    test('rejects a signature from a different key', () async {
      final otherKey = await Ed25519().newKeyPair();
      final body = utf8.encode('payload');
      final sig = await Ed25519().sign(body, keyPair: otherKey);
      final ok = await ManifestSignature.verify(body, base64Encode(sig.bytes));
      expect(ok, isFalse);
    });
  });
}
