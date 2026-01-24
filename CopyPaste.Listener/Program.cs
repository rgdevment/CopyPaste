using CopyPaste.Core;
using CopyPaste.Listener;

// 1. Initialize infrastructure
StorageConfig.Initialize();

// 2. Setup dependencies
var repository = new LiteDbRepository(StorageConfig.DatabasePath);
var service = new ClipboardService(repository);

// 3. Start listening
Console.WriteLine("CopyPaste Listener active...");
var listener = new WindowsClipboardListener(service);
listener.Run(); // keeps the app alive
