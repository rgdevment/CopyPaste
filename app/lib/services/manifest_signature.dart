import 'dart:convert';

import 'package:cryptography/cryptography.dart';
import 'package:flutter/foundation.dart' show visibleForTesting;
import 'package:flutter/services.dart' show rootBundle;

class ManifestSignature {
  ManifestSignature._();

  static const _pubKeyAsset = 'assets/keys/release_pubkey.txt';
  static final Ed25519 _algorithm = Ed25519();

  static SimplePublicKey? _cachedPublicKey;
  static SimplePublicKey? _overridePublicKey;

  static Future<bool> verify(List<int> bytes, String signatureBase64) async {
    try {
      final pub = await _loadPublicKey();
      if (pub == null) return false;
      final sigBytes = base64.decode(signatureBase64.trim());
      final signature = Signature(sigBytes, publicKey: pub);
      return await _algorithm.verify(bytes, signature: signature);
    } catch (_) {
      return false;
    }
  }

  static Future<SimplePublicKey?> _loadPublicKey() async {
    if (_overridePublicKey != null) return _overridePublicKey;
    if (_cachedPublicKey != null) return _cachedPublicKey;
    try {
      final raw = await rootBundle.loadString(_pubKeyAsset);
      final pubBytes = base64.decode(raw.trim());
      _cachedPublicKey = SimplePublicKey(pubBytes, type: KeyPairType.ed25519);
      return _cachedPublicKey;
    } catch (_) {
      return null;
    }
  }

  @visibleForTesting
  static void overridePublicKey(List<int> publicKeyBytes) {
    _overridePublicKey = SimplePublicKey(
      publicKeyBytes,
      type: KeyPairType.ed25519,
    );
  }

  @visibleForTesting
  static void reset() {
    _cachedPublicKey = null;
    _overridePublicKey = null;
  }
}
