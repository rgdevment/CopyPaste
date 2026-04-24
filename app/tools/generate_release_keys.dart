import 'dart:convert';
import 'dart:io';

import 'package:cryptography/cryptography.dart';
import 'package:path/path.dart' as p;

Future<void> main() async {
  final algorithm = Ed25519();
  final keyPair = await algorithm.newKeyPair();
  final pub = await keyPair.extractPublicKey();
  final priv = await keyPair.extractPrivateKeyBytes();

  final pubB64 = base64Encode(pub.bytes);
  final privB64 = base64Encode(priv);

  final repoRoot = Directory.current.path;
  final pubFile = File(
    p.join(repoRoot, 'app', 'assets', 'keys', 'release_pubkey.txt'),
  );
  final distDir = Directory(p.join(repoRoot, 'dist'));
  if (!distDir.existsSync()) distDir.createSync(recursive: true);
  final privFile = File(p.join(distDir.path, 'release_privkey.txt'));

  pubFile.parent.createSync(recursive: true);
  pubFile.writeAsStringSync('$pubB64\n');
  privFile.writeAsStringSync('$privB64\n');

  if (!Platform.isWindows) {
    Process.runSync('chmod', ['600', privFile.path]);
  }

  // ignore: avoid_print
  print('Public  key: ${pubFile.path}');
  // ignore: avoid_print
  print('Private key: ${privFile.path}');
  // ignore: avoid_print
  print('');
  // ignore: avoid_print
  print('Upload the private key as the GitHub secret RELEASE_PRIVATE_KEY:');
  // ignore: avoid_print
  print('  gh secret set RELEASE_PRIVATE_KEY < ${privFile.path}');
}
