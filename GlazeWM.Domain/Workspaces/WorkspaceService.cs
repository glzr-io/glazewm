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

    private readonly Stack<Workspace> _recentWorkspaces = new();
    public Workspace MostRecentWorkspace { get; set; }

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

    public IEnumerable<WorkspaceConfig> GetInactiveWorkspaceConfigs()
    {
      var activeWorkspaces = GetActiveWorkspaces();

      return _userConfigService.WorkspaceConfigs.Where(
        (config) => !activeWorkspaces.Any((workspace) => workspace.Name == config.Name)
      );
    }

    public WorkspaceConfig GetWorkspaceConfigToActivate(Monitor monitor)
    {
      var inactiveWorkspaceConfigs = GetInactiveWorkspaceConfigs();
      var boundWorkspaceConfig = inactiveWorkspaceConfigs
        .FirstOrDefault(config => config.BindToMonitor == monitor.DeviceName);

      if (boundWorkspaceConfig is not null)
        return boundWorkspaceConfig;

      var unreservedWorkspaceConfig = inactiveWorkspaceConfigs
        .FirstOrDefault(config => string.IsNullOrWhiteSpace(config.BindToMonitor));

      if (unreservedWorkspaceConfig is not null)
        return unreservedWorkspaceConfig;

      return inactiveWorkspaceConfigs.ElementAtOrDefault(0);
    }

    public static Workspace GetWorkspaceFromChildContainer(Container container)
    {
      return container.SelfAndAncestors.OfType<Workspace>().FirstOrDefault();
    }

    public Workspace GetFocusedWorkspace()
    {
      return GetWorkspaceFromChildContainer(_containerService.FocusedContainer);
    }

    public Workspace PopRecentWorkspace()
    {
      if (_recentWorkspaces.Count > 0)
      {
        return _recentWorkspaces.Pop();
      }
      return null;
    }

    public void PushRecentWorkspace(Workspace workspace)
    {
      _recentWorkspaces.Push(workspace);
    }
  }
}
