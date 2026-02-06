// This file is used by Code Analysis to maintain SuppressMessage
// attributes that are applied to this project.
// Project-level suppressions either have no target or are given
// a specific target and scoped to a namespace, type, member, etc.

using System.Diagnostics.CodeAnalysis;

[assembly: SuppressMessage("Design", "CA1031:Don't catch general exception types.", Justification = "Cleanup operations should not throw - log and continue", Scope = "member", Target = "~M:CopyPaste.Core.SqliteRepository.CleanupItemFiles(CopyPaste.Core.ClipboardItem)")]
[assembly: SuppressMessage("Design", "CA1031:Don't catch general exception types.", Justification = "Database corruption recovery must catch all exceptions to attempt backup", Scope = "member", Target = "~M:CopyPaste.Core.SqliteRepository.HandleCorruptDatabase")]
[assembly: SuppressMessage("Design", "CA1031:Don't catch general exception types.", Justification = "Dispose should never throw exceptions", Scope = "member", Target = "~M:CopyPaste.Core.SqliteRepository.Dispose")]
