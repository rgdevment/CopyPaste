import 'dart:convert';
import 'dart:io';

import 'package:cryptography/cryptography.dart';

Future<void> main(List<String> args) async {
  if (args.length < 2) {
    stderr.writeln(
      'Usage: dart run app/tools/sign_manifest.dart <input.json> <output.sig>',
    );
    stderr.writeln('Reads the base64 Ed25519 private key from stdin.');
    exit(64);
  }

  final inputPath = args[0];
  final outputPath = args[1];

  final inputFile = File(inputPath);
  if (!inputFile.existsSync()) {
    stderr.writeln('Input file not found: $inputPath');
    exit(66);
  }

  final privKeyB64 = stdin
      .transform(utf8.decoder)
      .transform(const LineSplitter())
      .where((l) => l.trim().isNotEmpty);
  final privLines = await privKeyB64.toList();
  if (privLines.isEmpty) {
    stderr.writeln('No private key on stdin.');
    exit(65);
  }
  final privBytes = base64Decode(privLines.first.trim());

  final algorithm = Ed25519();
  final keyPair = await algorithm.newKeyPairFromSeed(privBytes);
  final bytes = await inputFile.readAsBytes();
  final signature = await algorithm.sign(bytes, keyPair: keyPair);

  final sigB64 = base64Encode(signature.bytes);
  await File(outputPath).writeAsString('$sigB64\n');

  // ignore: avoid_print
  print('Signed $inputPath -> $outputPath');
}
