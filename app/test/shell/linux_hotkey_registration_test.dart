import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/linux_hotkey_registration.dart';

class _FakeLinuxHotkeyBindingApi implements LinuxHotkeyBindingApi {
  _FakeLinuxHotkeyBindingApi(this.responses);

  final List<bool> responses;
  final List<HotkeyBinding> attempts = <HotkeyBinding>[];

  @override
  Future<bool> registerHotkey(HotkeyBinding binding) async {
    attempts.add(binding);
    if (responses.isEmpty) return false;
    return responses.removeAt(0);
  }
}

void main() {
  group('registerLinuxHotkeyWithFallback', () {
    const requested = HotkeyBinding(
      virtualKey: 0x56,
      keyName: 'V',
      useCtrl: true,
      useWin: false,
      useAlt: true,
      useShift: false,
    );

    test('registers requested binding when available', () async {
      final api = _FakeLinuxHotkeyBindingApi(<bool>[true]);

      final result = await registerLinuxHotkeyWithFallback(
        api: api,
        requestedBinding: requested,
      );

      expect(result.status, HotkeyRegistrationStatus.registered);
      expect(result.effectiveBinding, requested);
      expect(api.attempts, <HotkeyBinding>[requested]);
    });

    test(
      'falls back to temporary Linux shortcut when requested binding fails',
      () async {
        final api = _FakeLinuxHotkeyBindingApi(<bool>[false, true]);

        final result = await registerLinuxHotkeyWithFallback(
          api: api,
          requestedBinding: requested,
        );

        expect(result.status, HotkeyRegistrationStatus.fallbackRegistered);
        expect(result.effectiveBinding, kLinuxTemporaryFallbackHotkey);
        expect(api.attempts, <HotkeyBinding>[
          requested,
          kLinuxTemporaryFallbackHotkey,
        ]);
      },
    );

    test(
      'fails cleanly when requested and temporary fallback both fail',
      () async {
        final api = _FakeLinuxHotkeyBindingApi(<bool>[false, false]);

        final result = await registerLinuxHotkeyWithFallback(
          api: api,
          requestedBinding: requested,
        );

        expect(result.status, HotkeyRegistrationStatus.failed);
        expect(result.effectiveBinding, isNull);
        expect(api.attempts, <HotkeyBinding>[
          requested,
          kLinuxTemporaryFallbackHotkey,
        ]);
      },
    );

    test(
      'does not retry when requested binding already matches temporary fallback',
      () async {
        final api = _FakeLinuxHotkeyBindingApi(<bool>[false]);

        final result = await registerLinuxHotkeyWithFallback(
          api: api,
          requestedBinding: kLinuxTemporaryFallbackHotkey,
        );

        expect(result.status, HotkeyRegistrationStatus.failed);
        expect(api.attempts, <HotkeyBinding>[kLinuxTemporaryFallbackHotkey]);
      },
    );
  });
}
