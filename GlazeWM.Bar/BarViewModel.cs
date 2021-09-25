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
    public string Padding { get; set; }
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
      BorderWidth = ShorthandToXamlProperty(_barConfig.BorderWidth);
      Padding = ShorthandToXamlProperty(_barConfig.Padding);

      UpdateWorkspaces();
    }

    /// <summary>
    /// Convert shorthand properties from user config (ie. `Padding`, `Margin`, and `BorderWidth`)
    /// to be compatible with their equivalent XAML properties (ie. `Padding`, `Margin`, and
    /// `BorderThickness`). Shorthand properties follow the 1-to-4 value syntax used in CSS.
    /// </summary>
    private string ShorthandToXamlProperty(string shorthand)
    {
      var shorthandParts = shorthand.Split(" ");

      return shorthandParts.Count() switch
      {
        1 => shorthand,
        2 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[0]}",
        3 => $"{shorthandParts[1]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
        4 => $"{shorthandParts[3]},{shorthandParts[0]},{shorthandParts[1]},{shorthandParts[2]}",
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
