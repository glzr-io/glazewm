using System.Collections.ObjectModel;
using System.Windows.Threading;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace GlazeWM.Bar.Components
{
  public class WorkspacesComponentViewModel : ComponentViewModel
  {
    private WorkspaceService _workspaceService = ServiceLocator.Provider.GetRequiredService<WorkspaceService>();
    private readonly BarViewModel _barViewModel;
    public ObservableCollection<Workspace> Workspaces => _barViewModel.Workspaces;
    private Dispatcher _dispatcher => _barViewModel.Dispatcher;
    private Monitor _monitor => _barViewModel.Monitor;

    public WorkspacesComponentViewModel(BarViewModel parentViewModel) : base(parentViewModel)
    {
    }
  }
}
