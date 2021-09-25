using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using System.Windows.Threading;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;

namespace GlazeWM.Bar
{
  public class BarViewModel : INotifyPropertyChanged
  {
    public event PropertyChangedEventHandler PropertyChanged;
    public ObservableCollection<Workspace> Workspaces { get; set; } = new ObservableCollection<Workspace>();
    public string Background { get; set; }
    public string FontFamily { get; set; }
    public string FontSize { get; set; }
    public string BorderColor { get; set; }
    public string BorderWidth { get; set; }
    private readonly Dispatcher _dispatcher;
    private readonly Monitor _monitor;
    private readonly BarConfig _barConfig;

    public BarViewModel(Dispatcher dispatcher, Monitor monitor, BarConfig barConfig)
    {
      _dispatcher = dispatcher;
      _monitor = monitor;
      _barConfig = barConfig;
    }

    public void InitializeState()
    {
      Background = _barConfig.Background;
      FontFamily = _barConfig.FontFamily;
      FontSize = _barConfig.FontSize;
      BorderColor = _barConfig.BorderColor;
      BorderWidth = FormatBorderWidth(_barConfig.BorderWidth);

      UpdateWorkspaces();
    }

    /// <summary>
    /// Format border width from user config to be compatible with `BorderThickness`.
    /// </summary>
    private string FormatBorderWidth(string borderWidth)
    {
      var borderWidthParts = borderWidth.Split(" ");

      return borderWidthParts.Count() switch
      {
        1 => borderWidth,
        2 => $"{borderWidthParts[1]},{borderWidthParts[0]},{borderWidthParts[1]},{borderWidthParts[0]}",
        3 => $"{borderWidthParts[1]},{borderWidthParts[0]},{borderWidthParts[1]},{borderWidthParts[2]}",
        4 => $"{borderWidthParts[3]},{borderWidthParts[0]},{borderWidthParts[1]},{borderWidthParts[2]}",
        _ => throw new ArgumentException(),
      };
    }

    public void UpdateWorkspaces()
    {
      _dispatcher.Invoke(() =>
      {
        Workspaces.Clear();

        foreach (var workspace in _monitor.Children)
          Workspaces.Add(workspace as Workspace);
      });
    }

    private void OnPropertyChanged(string propertyName)
    {
      PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
  }
}
