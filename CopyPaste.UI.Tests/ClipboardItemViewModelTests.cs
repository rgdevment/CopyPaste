using CopyPaste.Core;
using Xunit;

namespace CopyPaste.UI.Tests;

public class ClipboardItemViewModelTests
{
    #region Constructor Tests

    [Fact]
    public void Constructor_WithValidModel_CreatesInstance()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.NotNull(viewModel);
        Assert.Equal(model, viewModel.Model);
    }

    [Fact]
    public void Constructor_WithNullModel_ThrowsException()
    {
        Assert.Throws<ArgumentNullException>(() =>
            new ViewModels.ClipboardItemViewModel(
                null!,
                _ => { },
                (_, _) => { },
                _ => { }
            )
        );
    }

    [Fact]
    public void Constructor_InitializesIsPinnedFromModel()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            IsPinned = true
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.True(viewModel.IsPinned);
    }

    #endregion

    #region IsPinned Tests

    [Fact]
    public void IsPinned_WhenChanged_UpdatesModel()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            IsPinned = false
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        viewModel.IsPinned = true;

        Assert.True(model.IsPinned);
    }

    [Fact]
    public void IsPinned_WhenSetToSameValue_DoesNotRaisePropertyChanged()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            IsPinned = false
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        var changeCount = 0;
        viewModel.PropertyChanged += (_, _) => changeCount++;

        viewModel.IsPinned = false;

        Assert.Equal(0, changeCount);
    }

    #endregion

    #region Timestamp Tests

    [Fact]
    public void Timestamp_Today_ShowsHoy()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.StartsWith("Hoy", viewModel.Timestamp);
    }

    [Fact]
    public void Timestamp_Yesterday_ShowsAyer()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow.AddDays(-1)
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.StartsWith("Ayer", viewModel.Timestamp);
    }

    [Fact]
    public void Timestamp_OlderDate_ShowsFormattedDate()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            CreatedAt = DateTime.UtcNow.AddDays(-7)
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.DoesNotContain("Hoy", viewModel.Timestamp);
        Assert.DoesNotContain("Ayer", viewModel.Timestamp);
    }

    #endregion

    #region RefreshFromModel Tests

    [Fact]
    public void RefreshFromModel_UpdatesContent()
    {
        var model = new ClipboardItem
        {
            Content = "Original",
            Type = ClipboardContentType.Text
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        var updatedModel = new ClipboardItem
        {
            Content = "Updated",
            Type = ClipboardContentType.Text
        };

        viewModel.RefreshFromModel(updatedModel);

        Assert.Equal("Updated", viewModel.Model.Content);
    }

    [Fact]
    public void RefreshFromModel_WithNullModel_ThrowsException()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.Throws<ArgumentNullException>(() => viewModel.RefreshFromModel(null!));
    }

    [Fact]
    public void RefreshFromModel_UpdatesMetadata()
    {
        var model = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            Metadata = null
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        var updatedModel = new ClipboardItem
        {
            Content = "Test",
            Type = ClipboardContentType.Text,
            Metadata = "{\"key\":\"value\"}"
        };

        viewModel.RefreshFromModel(updatedModel);

        Assert.Equal("{\"key\":\"value\"}", viewModel.Model.Metadata);
    }

    #endregion

    #region Model Property Tests

    [Fact]
    public void Model_ReturnsOriginalModel()
    {
        var model = new ClipboardItem
        {
            Id = Guid.NewGuid(),
            Content = "Test",
            Type = ClipboardContentType.Text
        };

        var viewModel = new ViewModels.ClipboardItemViewModel(
            model,
            _ => { },
            (_, _) => { },
            _ => { }
        );

        Assert.Same(model, viewModel.Model);
        Assert.Equal(model.Id, viewModel.Model.Id);
    }

    #endregion
}

/*
 * SUGERENCIAS PARA TESTING ADICIONAL DE ClipboardItemViewModel:
 * 
 * 1. Command Tests (requieren setup específico de WinUI):
 *    - CopyCommand: Verificar que invoque _pasteAction correctamente
 *    - CopyAndCloseCommand: Verificar que invoque con closeAfter=true
 *    - DeleteCommand: Verificar que invoque _deleteAction
 *    - TogglePinCommand: Verificar que invoque _pinAction
 * 
 * 2. Property Tests Avanzados:
 *    - Content: Probar diferentes tipos de contenido (texto, rutas de archivo)
 *    - ThumbnailPath: Verificar parsing de metadata para thumbnails
 *    - ImagePath: Verificar parsing de rutas de imagen
 *    - MediaDuration: Verificar formateo de duración desde metadata
 *    - ImageDimensions: Verificar formateo de dimensiones desde metadata
 *    - FileSize: Verificar formateo de tamaño de archivo
 * 
 * 3. Visibility Properties:
 *    - ImageVisibility: Probar con diferentes tipos de contenido
 *    - MediaThumbnailVisibility: Verificar para audio/video
 *    - DurationVisibility: Verificar para media con duración
 *    - ImageDimensionsVisibility: Verificar para imágenes con dimensiones
 *    - FileSizeVisibility: Verificar para archivos con tamaño
 * 
 * 4. Icon/Display Properties:
 *    - TypeIcon: Verificar íconos correctos para cada ClipboardContentType
 *    - PinIconGlyph: Verificar cambio basado en IsPinned
 *    - PinMenuText: Verificar texto correcto según estado
 * 
 * 5. Integration Tests:
 *    - Probar interacción con ClipboardService real
 *    - Verificar actualización de UI cuando cambia thumbnail
 *    - Probar flujo completo de pin/unpin
 *    - Verificar eliminación y actualización de colección padre
 */
