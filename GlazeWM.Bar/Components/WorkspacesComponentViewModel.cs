using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces;

namespace GlazeWM.Bar
{
  public class WorkspacesComponentViewModel : ViewModelBase
  {
    public ObservableCollection<Workspace> Workspaces { get; set; } = new ObservableCollection<Workspace>();
    private readonly BarViewModel _barViewModel;
    private readonly WorkspaceService _workspaceService;
    private Dispatcher _dispatcher => _barViewModel.Dispatcher;
    private Monitor _monitor => _barViewModel.Monitor;

    public WorkspacesComponentViewModel(BarViewModel barViewModel, WorkspaceService workspaceService)
    {
      _barViewModel = barViewModel;
      _workspaceService = workspaceService;
    }

    public void InitializeState()
    {
      UpdateWorkspaces();
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
  }
}
