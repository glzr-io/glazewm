using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using System.Collections.Generic;
using System.Linq;

namespace GlazeWM.Domain.Workspaces
{
  public class WorkspaceService
  {
    private ContainerService _containerService;
    private UserConfigService _userConfigService;

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

    public IEnumerable<string> GetInactiveWorkspaceNames()
    {
      var activeWorkspaces = GetActiveWorkspaces();

      var inactiveWorkspaceConfigs = _userConfigService.UserConfig.Workspaces.Where(
        (config) => !activeWorkspaces.Any((workspace) => workspace.Name == config.Name)
      );

      return inactiveWorkspaceConfigs.Select(config => config.Name);
    }

    public Workspace GetWorkspaceFromChildContainer(Container container)
    {
      return container.SelfAndAncestors.OfType<Workspace>().FirstOrDefault();
    }

    public Workspace GetFocusedWorkspace()
    {
      return GetWorkspaceFromChildContainer(_containerService.FocusedContainer);
    }
  }
}
