import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:listener/listener.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  testWidgets('WindowsClipboardListener can be instantiated',
      (WidgetTester tester) async {
    final instance = WindowsClipboardListener();
    expect(instance, isNotNull);
    expect(instance.onEvent, isNotNull);
  });
}
