using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Windows.Threading;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;

namespace GlazeWM.Bar
{
  public class BarViewModel : INotifyPropertyChanged
  {
    public event PropertyChangedEventHandler PropertyChanged;
    private readonly Dispatcher _dispatcher;
    private readonly Monitor _monitor;
    private readonly BarConfig _barConfig;

    public BarViewModel(Dispatcher dispatcher, Monitor monitor, BarConfig barConfig)
    {
      _dispatcher = dispatcher;
      _monitor = monitor;
      _barConfig = barConfig;
    }

    private ObservableCollection<Workspace> _workspaces = new ObservableCollection<Workspace>();
    public ObservableCollection<Workspace> Workspaces
    {
      get { return _workspaces; }
      set
      {
        _workspaces = value;
        OnPropertyChanged("Workspaces");
      }
    }

    public void SetWorkspaces()
    {
      _dispatcher.Invoke(() =>
      {
        Workspaces.Clear();

        foreach (var workspace in _monitor.Children)
          Workspaces.Add(workspace as Workspace);

        OnPropertyChanged("Workspaces");
      });
    }

    private void OnPropertyChanged(string propertyName)
    {
      PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
  }
}
