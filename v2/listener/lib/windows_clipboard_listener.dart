import 'package:flutter/services.dart';

import 'clipboard_event.dart';

class ClipboardListener {
  static const EventChannel _channel = EventChannel('copypaste/clipboard');

  late final Stream<ClipboardEvent> onEvent = _channel
      .receiveBroadcastStream()
      .map((dynamic event) {
        final map = Map<Object?, Object?>.from(event as Map);
        return ClipboardEvent.fromMap(map);
      });
}
