using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Monitors;

namespace GlazeWM.Domain.Workspaces
{
  public class WorkspaceService
  {
    private readonly ContainerService _containerService;
    private readonly UserConfigService _userConfigService;

    public WorkspaceService(ContainerService containerService, UserConfigService userConfigService)
    {
      _containerService = containerService;
      _userConfigService = userConfigService;
    }

    /// <summary>
    /// Get active workspaces by iterating over the 2nd level of container tree.
    /// </summary>
    public IEnumerable<Workspace> GetActiveWorkspaces()
    {
      return _containerService.ContainerTree.Children
        .SelectMany(monitor => monitor.Children)
        .Cast<Workspace>();
    }

    public Workspace GetActiveWorkspaceByName(string name)
    {
      return GetActiveWorkspaces().FirstOrDefault(workspace => workspace.Name == name);
    }

    private IEnumerable<string> GetInactiveWorkspaceNames()
    {
      var activeWorkspaces = GetActiveWorkspaces();

      var inactiveWorkspaceConfigs = _userConfigService.UserConfig.Workspaces.Where(
        (config) => !activeWorkspaces.Any((workspace) => workspace.Name == config.Name)
      );

      return inactiveWorkspaceConfigs.Select(config => config.Name);
    }
    
    public string GetInactiveWorkspaceNameForMonitor(Monitor monitor)
    {
      return GetInactiveWorkspaceNames()
        .FirstOrDefault(w =>
          _userConfigService.UserConfig.Workspaces.First(uw => uw.Name == w).BindToMonitor == monitor.DeviceName);
    }

    public IEnumerable<string> GetInactiveWorkspaceNamesNotDedicatedToAMonitor()
    {
      return GetInactiveWorkspaceNames()
        .Where(w => string.IsNullOrWhiteSpace(_userConfigService.UserConfig.Workspaces.First(uw => uw.Name == w)
          .BindToMonitor));
    }

    public static Workspace GetWorkspaceFromChildContainer(Container container)
    {
      return container.SelfAndAncestors.OfType<Workspace>().FirstOrDefault();
    }

    public Workspace GetFocusedWorkspace()
    {
      return GetWorkspaceFromChildContainer(_containerService.FocusedContainer);
    }
  }
}
