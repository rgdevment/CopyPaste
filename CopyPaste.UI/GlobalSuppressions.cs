// This file is used by Code Analysis to maintain SuppressMessage
// attributes that are applied to this project.
// Project-level suppressions either have no target or are given
// a specific target and scoped to a namespace, type, member, etc.

using System.Diagnostics.CodeAnalysis;

[assembly: SuppressMessage("Design", "CA1031:No capture tipos de excepción generales.", Justification = "Image loading failures should be handled gracefully without crashing the UI", Scope = "member", Target = "~M:CopyPaste.UI.MainWindow.LoadImageSource(Microsoft.UI.Xaml.Controls.Image,System.String)")]
