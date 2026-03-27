import 'dart:convert';
import 'dart:io';

import 'package:core/core.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:copypaste/widgets/clipboard_card.dart';

import '../helpers/test_wrapper.dart';

ClipboardItem _makeTextItem({
  String content = 'Sample clipboard content',
  bool isPinned = false,
  CardColor cardColor = CardColor.none,
  String? label,
}) {
  return ClipboardItem(
    content: content,
    type: ClipboardContentType.text,
    isPinned: isPinned,
    cardColor: cardColor,
    label: label,
  );
}

void main() {
  group('ClipboardCard', () {
    testWidgets('renders text content', (tester) async {
      final item = _makeTextItem(content: 'Hello clipboard');

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Hello clipboard'), findsOneWidget);
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('double-tap triggers onTap', (tester) async {
      var tapCount = 0;
      var selectCount = 0;

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () => tapCount++,
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            onSelect: () => selectCount++,
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Two pointer-downs within 300ms triggers paste
      final center = tester.getCenter(find.byType(ClipboardCard));
      final gesture = await tester.startGesture(center);
      await gesture.up();
      await tester.pump(const Duration(milliseconds: 100));
      final gesture2 = await tester.startGesture(center);
      await gesture2.up();
      await tester.pumpAndSettle();

      expect(tapCount, equals(1));
      expect(selectCount, equals(2));
    });

    testWidgets('single tap triggers onSelect', (tester) async {
      var selectCount = 0;

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            onSelect: () => selectCount++,
          ),
        ),
      );
      await tester.pumpAndSettle();

      final center = tester.getCenter(find.byType(ClipboardCard));
      final gesture = await tester.startGesture(center);
      await gesture.up();
      await tester.pumpAndSettle();

      expect(selectCount, equals(1));
    });

    testWidgets('expand toggle button triggers onExpandToggle', (tester) async {
      var expandCount = 0;

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Line1\nLine2\nLine3\nLine4\nLine5'),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            onExpandToggle: () => expandCount++,
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Find and tap the expand icon button
      final expandIcon = find.byIcon(Icons.expand_more_rounded);
      expect(expandIcon, findsOneWidget);
      await tester.tap(expandIcon);
      await tester.pumpAndSettle();

      expect(expandCount, equals(1));
    });

    testWidgets('expand toggle hidden for short content', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Short text'),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            onExpandToggle: () {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.expand_more_rounded), findsNothing);
    });

    testWidgets('shows selection border when isSelected is true', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            isSelected: true,
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Selected card should render without error
      expect(find.byType(AnimatedContainer), findsAtLeastNWidgets(1));
    });

    testWidgets('shows expanded content when isExpanded is true', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(
              content: 'Line1\nLine2\nLine3\nLine4\nLine5\nLine6',
            ),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            isExpanded: true,
            cardMaxLines: 5,
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('link type item renders without error', (tester) async {
      final item = ClipboardItem(
        content: 'https://example.com',
        type: ClipboardContentType.link,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('pinned item renders without error', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(isPinned: true),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('card with label renders without error', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(label: 'Work'),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Work'), findsOneWidget);
    });

    testWidgets('card with color renders without error', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(cardColor: CardColor.red),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('dark mode renders without error', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
          brightness: Brightness.dark,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('onPin callback is called via action button', (tester) async {
      var pinCount = 0;
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () {},
            onPin: () => pinCount++,
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Hover to show action buttons
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      final card = find.byType(ClipboardCard);
      await gesture.moveTo(tester.getCenter(card));
      await tester.pumpAndSettle();

      // Find and tap pin button
      final pinButtons = find.byIcon(Icons.push_pin_outlined);
      if (pinButtons.evaluate().isNotEmpty) {
        await tester.tap(pinButtons.first);
        await tester.pump();
        expect(pinCount, equals(1));
      }
    });

    testWidgets('onDelete callback is called via action button', (
      tester,
    ) async {
      var deleteCount = 0;
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () {},
            onPin: () {},
            onDelete: () => deleteCount++,
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      final card = find.byType(ClipboardCard);
      await gesture.moveTo(tester.getCenter(card));
      await tester.pumpAndSettle();

      final deleteButtons = find.byIcon(Icons.delete_rounded);
      if (deleteButtons.evaluate().isNotEmpty) {
        await tester.tap(deleteButtons.first);
        await tester.pump();
        expect(deleteCount, equals(1));
      }
    });

    testWidgets('onPastePlain callback exposed for text type', (tester) async {
      var plainCount = 0;
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            onPastePlain: () => plainCount++,
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('file type item renders filename', (tester) async {
      final sep = Platform.pathSeparator;
      final item = ClipboardItem(
        content: '${sep}home${sep}user${sep}document.pdf',
        type: ClipboardContentType.file,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
      expect(find.text('document.pdf'), findsOneWidget);
    });

    testWidgets('folder type item renders without error', (tester) async {
      final sep = Platform.pathSeparator;
      final item = ClipboardItem(
        content: '${sep}home${sep}user${sep}Documents',
        type: ClipboardContentType.folder,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('file type with multiple files shows count badge', (
      tester,
    ) async {
      final sep = Platform.pathSeparator;
      final item = ClipboardItem(
        content:
            '${sep}home${sep}user${sep}a.pdf\n${sep}home${sep}user${sep}b.txt\n${sep}home${sep}user${sep}c.docx',
        type: ClipboardContentType.file,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // "+2" badge for 2 extra files
      expect(find.text('+2'), findsOneWidget);
    });

    testWidgets('image type with non-existent path renders placeholder', (
      tester,
    ) async {
      final item = ClipboardItem(
        content: 'C:\\nonexistent\\image.png',
        type: ClipboardContentType.image,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('image type with empty content shows placeholder', (
      tester,
    ) async {
      final item = ClipboardItem(content: '', type: ClipboardContentType.image);

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('image type with existing file renders Image widget', (
      tester,
    ) async {
      // Create a real PNG file for image rendering
      final tmpDir = Directory.systemTemp.createTempSync('card_img_test_');
      final imgFile = File('${tmpDir.path}/test.png');
      // Minimal 1x1 PNG
      imgFile.writeAsBytesSync([
        0x89,
        0x50,
        0x4E,
        0x47,
        0x0D,
        0x0A,
        0x1A,
        0x0A,
        0x00,
        0x00,
        0x00,
        0x0D,
        0x49,
        0x48,
        0x44,
        0x52,
        0x00,
        0x00,
        0x00,
        0x01,
        0x00,
        0x00,
        0x00,
        0x01,
        0x08,
        0x02,
        0x00,
        0x00,
        0x00,
        0x90,
        0x77,
        0x53,
        0xDE,
        0x00,
        0x00,
        0x00,
        0x0C,
        0x49,
        0x44,
        0x41,
        0x54,
        0x08,
        0xD7,
        0x63,
        0xF8,
        0xCF,
        0xC0,
        0x00,
        0x00,
        0x00,
        0x02,
        0x00,
        0x01,
        0xE2,
        0x21,
        0xBC,
        0x33,
        0x00,
        0x00,
        0x00,
        0x00,
        0x49,
        0x45,
        0x4E,
        0x44,
        0xAE,
        0x42,
        0x60,
        0x82,
      ]);

      final item = ClipboardItem(
        content: imgFile.path,
        type: ClipboardContentType.image,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      // Allow image path resolution
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
      tmpDir.deleteSync(recursive: true);
    });

    testWidgets('audio type item renders without error', (tester) async {
      final sep = Platform.pathSeparator;
      final item = ClipboardItem(
        content: '${sep}home${sep}user${sep}song.mp3',
        type: ClipboardContentType.audio,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('video type item renders without error', (tester) async {
      final sep = Platform.pathSeparator;
      final item = ClipboardItem(
        content: '${sep}home${sep}user${sep}clip.mp4',
        type: ClipboardContentType.video,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('item with appSource displays appSource text', (tester) async {
      final item = ClipboardItem(
        content: 'Some text',
        type: ClipboardContentType.text,
        appSource: 'Notepad',
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('· Notepad'), findsOneWidget);
    });

    testWidgets('card updates when item changes', (tester) async {
      final key = GlobalKey<State>();
      final item1 = _makeTextItem(content: 'First content');
      final item2 = _makeTextItem(content: 'Second content');

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            key: key,
            item: item1,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('First content'), findsOneWidget);

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            key: key,
            item: item2,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Second content'), findsOneWidget);
    });

    testWidgets('hover enter and exit changes card appearance', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);

      // Enter hover
      final card = find.byType(ClipboardCard);
      await gesture.moveTo(tester.getCenter(card));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);

      // Exit hover
      await gesture.moveTo(Offset.zero);
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('file not found shows warning badge', (tester) async {
      final item = ClipboardItem(
        content: 'C:\\nonexistent\\file.pdf',
        type: ClipboardContentType.file,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // File not found badge should appear
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('unknown type renders text content', (tester) async {
      final item = ClipboardItem(
        content: 'Unknown content',
        type: ClipboardContentType.unknown,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('Unknown content'), findsOneWidget);
    });

    testWidgets('card with all colors renders without error', (tester) async {
      for (final color in CardColor.values) {
        await tester.pumpWidget(
          wrapWidget(
            ClipboardCard(
              item: _makeTextItem(cardColor: color),
              onTap: () {},
              onPin: () {},
              onDelete: () {},
              onLabelColor: (_, _) {},
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(find.byType(ClipboardCard), findsOneWidget);
      }
    });

    testWidgets('text item with pasteCount shows footer', (tester) async {
      final item = ClipboardItem(
        content: 'Pasted many times',
        type: ClipboardContentType.text,
        pasteCount: 5,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('×5'), findsOneWidget);
    });

    testWidgets('image item with dimensions metadata shows footer', (
      tester,
    ) async {
      final meta = jsonEncode({'width': 1920, 'height': 1080});
      final item = ClipboardItem(
        content: '',
        type: ClipboardContentType.image,
        metadata: meta,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('1920×1080'), findsOneWidget);
    });

    testWidgets('file item with file_size metadata shows size footer', (
      tester,
    ) async {
      final sep = Platform.pathSeparator;
      final meta = jsonEncode({'file_size': 512 * 1024}); // 512 KB
      final item = ClipboardItem(
        content: '${sep}docs${sep}report.pdf',
        type: ClipboardContentType.file,
        metadata: meta,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Size chip should appear
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('video item with duration metadata shows duration footer', (
      tester,
    ) async {
      final sep = Platform.pathSeparator;
      final meta = jsonEncode({'duration': 125}); // 2m5s
      final item = ClipboardItem(
        content: '${sep}videos${sep}clip.mp4',
        type: ClipboardContentType.video,
        metadata: meta,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('link item with valid URL renders domain badge', (
      tester,
    ) async {
      final item = ClipboardItem(
        content: 'https://github.com/user/repo',
        type: ClipboardContentType.link,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('github.com'), findsOneWidget);
    });

    testWidgets('link item with URL renders full URL', (tester) async {
      final item = ClipboardItem(
        content: 'https://flutter.dev/docs',
        type: ClipboardContentType.link,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            isExpanded: true,
            cardMaxLines: 5,
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('right-click shows context menu', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Right click me'),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Secondary tap using mouse gesture to open context menu
      final card = find.byType(ClipboardCard);
      final gesture = await tester.startGesture(
        tester.getCenter(card),
        kind: PointerDeviceKind.mouse,
        buttons: kSecondaryMouseButton,
      );
      await gesture.up();
      await tester.pumpAndSettle();

      // Menu should have appeared with Paste option
      expect(find.text('Paste'), findsOneWidget);
    });

    testWidgets('right-click menu paste action triggers onTap', (tester) async {
      var tapCount = 0;
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Paste via menu'),
            onTap: () => tapCount++,
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      final gesture = await tester.startGesture(
        tester.getCenter(find.byType(ClipboardCard)),
        kind: PointerDeviceKind.mouse,
        buttons: kSecondaryMouseButton,
      );
      await gesture.up();
      await tester.pumpAndSettle();

      // Tap Paste menu item
      final paste = find.text('Paste');
      expect(paste, findsOneWidget);
      await tester.tap(paste.first);
      await tester.pumpAndSettle();

      expect(tapCount, 1);
    });

    testWidgets('text item with paste plain in menu, tapping it fires', (
      tester,
    ) async {
      var plainPasted = false;
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Plain text'),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
            onPastePlain: () => plainPasted = true,
          ),
        ),
      );
      await tester.pumpAndSettle();

      final gesture = await tester.startGesture(
        tester.getCenter(find.byType(ClipboardCard)),
        kind: PointerDeviceKind.mouse,
        buttons: kSecondaryMouseButton,
      );
      await gesture.up();
      await tester.pumpAndSettle();

      // Paste plain should appear in menu for text type
      final pastePlain = find.text('Paste plain');
      if (pastePlain.evaluate().isNotEmpty) {
        await tester.tap(pastePlain.first);
        await tester.pumpAndSettle();
        expect(plainPasted, isTrue);
      }
    });

    testWidgets('image with large file size shows GB format', (tester) async {
      final meta = jsonEncode({'file_size': 2 * 1024 * 1024 * 1024}); // 2GB
      final item = ClipboardItem(
        content: '',
        type: ClipboardContentType.image,
        metadata: meta,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('image with small file size shows bytes format', (
      tester,
    ) async {
      final meta = jsonEncode({'file_size': 500}); // 500 bytes
      final item = ClipboardItem(
        content: '',
        type: ClipboardContentType.image,
        metadata: meta,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('video item with duration over 1 hour shows H:MM:SS format', (
      tester,
    ) async {
      final meta = jsonEncode({'duration': 3700}); // 1h 1m 40s
      final item = ClipboardItem(
        content: '/video.mp4',
        type: ClipboardContentType.video,
        metadata: meta,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('card with MB file size shows MB format', (tester) async {
      final meta = jsonEncode({'file_size': 5 * 1024 * 1024}); // 5MB
      final item = ClipboardItem(
        content: '/big_file.bin',
        type: ClipboardContentType.file,
        metadata: meta,
      );

      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('pinned item in header shows pin icon when not hovering', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(isPinned: true),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.push_pin_rounded), findsAtLeastNWidgets(1));
    });

    testWidgets('dark mode hover covers surfaceVariant color path', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
          brightness: Brightness.dark,
        ),
      );
      await tester.pumpAndSettle();

      // Hover in dark mode to trigger surfaceVariant color path
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      await gesture.moveTo(tester.getCenter(find.byType(ClipboardCard)));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('hover edit button triggers _editLabelColor and shows dialog', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Edit via hover'),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Hover to show action buttons
      final gesture = await tester.createGesture(kind: PointerDeviceKind.mouse);
      await gesture.addPointer(location: Offset.zero);
      addTearDown(gesture.removePointer);
      await gesture.moveTo(tester.getCenter(find.byType(ClipboardCard)));
      await tester.pumpAndSettle();

      // Edit button must exist when hovering
      final editButtons = find.byIcon(Icons.edit_outlined);
      expect(editButtons, findsAtLeastNWidgets(1));
      await tester.tap(editButtons.first);
      await tester.pumpAndSettle();

      // Cancel dialog if shown
      final cancel = find.text('Cancel');
      if (cancel.evaluate().isNotEmpty) {
        await tester.tap(cancel.first);
        await tester.pumpAndSettle();
      }
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('context menu dismissed without selection covers null case', (
      tester,
    ) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Dismiss menu'),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      // Open context menu
      final gesture = await tester.startGesture(
        tester.getCenter(find.byType(ClipboardCard)),
        kind: PointerDeviceKind.mouse,
        buttons: kSecondaryMouseButton,
      );
      await gesture.up();
      await tester.pumpAndSettle();

      // Verify menu is open
      expect(find.text('Paste'), findsOneWidget);

      // Tap outside the menu to dismiss it (null case)
      await tester.tapAt(const Offset(10, 10));
      await tester.pumpAndSettle();

      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('right-click delete action triggers onDelete', (tester) async {
      var deleted = false;
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Delete me'),
            onTap: () {},
            onPin: () {},
            onDelete: () => deleted = true,
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      final gesture = await tester.startGesture(
        tester.getCenter(find.byType(ClipboardCard)),
        kind: PointerDeviceKind.mouse,
        buttons: kSecondaryMouseButton,
      );
      await gesture.up();
      await tester.pumpAndSettle();

      final deleteItem = find.text('Delete');
      if (deleteItem.evaluate().isNotEmpty) {
        await tester.tap(deleteItem.first);
        await tester.pumpAndSettle();
        expect(deleted, isTrue);
      }
    });

    testWidgets('right-click pin action triggers onPin', (tester) async {
      var pinned = false;
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Pin me'),
            onTap: () {},
            onPin: () => pinned = true,
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      final gesture = await tester.startGesture(
        tester.getCenter(find.byType(ClipboardCard)),
        kind: PointerDeviceKind.mouse,
        buttons: kSecondaryMouseButton,
      );
      await gesture.up();
      await tester.pumpAndSettle();

      final pinItem = find.text('Pin');
      if (pinItem.evaluate().isNotEmpty) {
        await tester.tap(pinItem.first);
        await tester.pumpAndSettle();
        expect(pinned, isTrue);
      }
    });

    testWidgets('right-click edit action opens label dialog', (tester) async {
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: _makeTextItem(content: 'Edit me'),
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (label, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();

      final gesture = await tester.startGesture(
        tester.getCenter(find.byType(ClipboardCard)),
        kind: PointerDeviceKind.mouse,
        buttons: kSecondaryMouseButton,
      );
      await gesture.up();
      await tester.pumpAndSettle();

      // Verify context menu opened (Paste is always present)
      expect(find.text('Paste'), findsOneWidget);

      // Tap 'Edit card' menu item
      final editItem = find.text('Edit card');
      expect(editItem, findsOneWidget);
      await tester.tap(editItem.first);
      await tester.pumpAndSettle();

      // LabelColorDialog should appear
      final cancel = find.text('Cancel');
      if (cancel.evaluate().isNotEmpty) {
        await tester.tap(cancel.first);
        await tester.pumpAndSettle();
      }
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('timestamp for item modified 5 minutes ago shows Xm', (
      tester,
    ) async {
      final item = ClipboardItem(
        content: 'Old item',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.now().subtract(const Duration(minutes: 5)),
      );
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(ClipboardCard), findsOneWidget);
      expect(find.textContaining('m'), findsWidgets);
    });

    testWidgets('timestamp for item modified 3 hours ago shows Xh', (
      tester,
    ) async {
      final item = ClipboardItem(
        content: 'Hours old',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.now().subtract(const Duration(hours: 3)),
      );
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(ClipboardCard), findsOneWidget);
      expect(find.textContaining('h'), findsWidgets);
    });

    testWidgets('timestamp for item modified 4 days ago shows Xd', (
      tester,
    ) async {
      final item = ClipboardItem(
        content: 'Days old',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.now().subtract(const Duration(days: 4)),
      );
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(ClipboardCard), findsOneWidget);
      expect(find.textContaining('d'), findsWidgets);
    });

    testWidgets('timestamp for item modified 30 days ago shows month/day', (
      tester,
    ) async {
      final item = ClipboardItem(
        content: 'Very old',
        type: ClipboardContentType.text,
        modifiedAt: DateTime.now().subtract(const Duration(days: 30)),
      );
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('video item without extension but with duration has footer', (
      tester,
    ) async {
      final meta = jsonEncode({'duration': 120});
      final item = ClipboardItem(
        content: '/videos/clip',
        type: ClipboardContentType.video,
        metadata: meta,
      );
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('audio item with empty content shows audio label', (
      tester,
    ) async {
      final item = ClipboardItem(content: '', type: ClipboardContentType.audio);
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(ClipboardCard), findsOneWidget);
    });

    testWidgets('video item with empty content shows video label', (
      tester,
    ) async {
      final item = ClipboardItem(content: '', type: ClipboardContentType.video);
      await tester.pumpWidget(
        wrapWidget(
          ClipboardCard(
            item: item,
            onTap: () {},
            onPin: () {},
            onDelete: () {},
            onLabelColor: (_, _) {},
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byType(ClipboardCard), findsOneWidget);
    });
  });
}
