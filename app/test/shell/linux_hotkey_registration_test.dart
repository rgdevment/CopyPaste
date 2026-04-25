import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/shell/linux_hotkey_registration.dart';
import 'package:copypaste/shell/linux_shell.dart';

class _FakeLinuxHotkeyBindingApi implements LinuxHotkeyBindingApi {
  _FakeLinuxHotkeyBindingApi(this.responses);

  final List<HotkeyRegisterResponse> responses;
  final List<HotkeyBinding> attempts = <HotkeyBinding>[];

  @override
  Future<HotkeyRegisterResponse> registerHotkey(HotkeyBinding binding) async {
    attempts.add(binding);
    if (responses.isEmpty) {
      return const HotkeyRegisterResponse(success: false, errorCode: 'unknown');
    }
    return responses.removeAt(0);
  }
}

HotkeyRegisterResponse _ok() => const HotkeyRegisterResponse(success: true);
HotkeyRegisterResponse _fail(String code) =>
    HotkeyRegisterResponse(success: false, errorCode: code);

void main() {
  group('isLinuxSupportedVirtualKey', () {
    test('accepts A-Z, 0-9, F-keys, navigation, symbols', () {
      expect(isLinuxSupportedVirtualKey(0x41), isTrue);
      expect(isLinuxSupportedVirtualKey(0x5A), isTrue);
      expect(isLinuxSupportedVirtualKey(0x30), isTrue);
      expect(isLinuxSupportedVirtualKey(0x70), isTrue);
      expect(isLinuxSupportedVirtualKey(0x87), isTrue);
      expect(isLinuxSupportedVirtualKey(0x20), isTrue);
      expect(isLinuxSupportedVirtualKey(0x25), isTrue);
      expect(isLinuxSupportedVirtualKey(0xC0), isTrue);
    });

    test('rejects unmapped virtual keys', () {
      expect(isLinuxSupportedVirtualKey(0x00), isFalse);
      expect(isLinuxSupportedVirtualKey(0x90), isFalse);
      expect(isLinuxSupportedVirtualKey(0xFF), isFalse);
    });
  });

  group('registerLinuxHotkeyWithFallback', () {
    const requested = HotkeyBinding(
      virtualKey: 0x56,
      keyName: 'V',
      useCtrl: true,
      useWin: false,
      useAlt: true,
      useShift: false,
    );

    test(
      'short-circuits when requested key is unsupported (no remote call)',
      () async {
        const unsupported = HotkeyBinding(
          virtualKey: 0x99,
          keyName: '?',
          useCtrl: true,
          useWin: false,
          useAlt: true,
          useShift: false,
        );
        final api = _FakeLinuxHotkeyBindingApi(<HotkeyRegisterResponse>[]);

        final result = await registerLinuxHotkeyWithFallback(
          api: api,
          requestedBinding: unsupported,
        );

        expect(result.status, HotkeyRegistrationStatus.failed);
        expect(result.failureReason, HotkeyFailureReason.unsupportedKey);
        expect(api.attempts, isEmpty);
      },
    );

    test('registers requested binding when available', () async {
      final api = _FakeLinuxHotkeyBindingApi(<HotkeyRegisterResponse>[_ok()]);

      final result = await registerLinuxHotkeyWithFallback(
        api: api,
        requestedBinding: requested,
      );

      expect(result.status, HotkeyRegistrationStatus.registered);
      expect(result.effectiveBinding, requested);
      expect(result.failureReason, isNull);
      expect(api.attempts, <HotkeyBinding>[requested]);
    });

    test('falls back when requested binding fails with grabFailed', () async {
      final api = _FakeLinuxHotkeyBindingApi(<HotkeyRegisterResponse>[
        _fail('grabFailed'),
        _ok(),
      ]);

      final result = await registerLinuxHotkeyWithFallback(
        api: api,
        requestedBinding: requested,
      );

      expect(result.status, HotkeyRegistrationStatus.fallbackRegistered);
      expect(result.effectiveBinding, kLinuxTemporaryFallbackHotkey);
      expect(result.failureReason, HotkeyFailureReason.grabFailed);
      expect(api.attempts, <HotkeyBinding>[
        requested,
        kLinuxTemporaryFallbackHotkey,
      ]);
    });

    test('fails cleanly when requested and fallback both fail', () async {
      final api = _FakeLinuxHotkeyBindingApi(<HotkeyRegisterResponse>[
        _fail('grabFailed'),
        _fail('grabFailed'),
      ]);

      final result = await registerLinuxHotkeyWithFallback(
        api: api,
        requestedBinding: requested,
      );

      expect(result.status, HotkeyRegistrationStatus.failed);
      expect(result.effectiveBinding, isNull);
      expect(result.failureReason, HotkeyFailureReason.grabFailed);
    });

    test(
      'does not retry when requested binding equals temporary fallback',
      () async {
        final api = _FakeLinuxHotkeyBindingApi(<HotkeyRegisterResponse>[
          _fail('grabFailed'),
        ]);

        final result = await registerLinuxHotkeyWithFallback(
          api: api,
          requestedBinding: kLinuxTemporaryFallbackHotkey,
        );

        expect(result.status, HotkeyRegistrationStatus.failed);
        expect(result.failureReason, HotkeyFailureReason.grabFailed);
        expect(api.attempts, <HotkeyBinding>[kLinuxTemporaryFallbackHotkey]);
      },
    );

    test('maps unknown error code to HotkeyFailureReason.unknown', () async {
      final api = _FakeLinuxHotkeyBindingApi(<HotkeyRegisterResponse>[
        _fail('something_weird'),
        _fail('something_weird'),
      ]);

      final result = await registerLinuxHotkeyWithFallback(
        api: api,
        requestedBinding: requested,
      );

      expect(result.failureReason, HotkeyFailureReason.unknown);
    });

    test('maps noModifier and noX11 error codes', () async {
      final api1 = _FakeLinuxHotkeyBindingApi(<HotkeyRegisterResponse>[
        _fail('noModifier'),
        _fail('noModifier'),
      ]);
      final r1 = await registerLinuxHotkeyWithFallback(
        api: api1,
        requestedBinding: requested,
      );
      expect(r1.failureReason, HotkeyFailureReason.noModifier);

      final api2 = _FakeLinuxHotkeyBindingApi(<HotkeyRegisterResponse>[
        _fail('noX11'),
        _fail('noX11'),
      ]);
      final r2 = await registerLinuxHotkeyWithFallback(
        api: api2,
        requestedBinding: requested,
      );
      expect(r2.failureReason, HotkeyFailureReason.noX11);
    });
  });
}
