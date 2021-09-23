using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using System.Collections.Generic;
using System.Linq;

namespace GlazeWM.Domain.Workspaces
{
  public class WorkspaceService
  {
    public List<Workspace> InactiveWorkspaces { get; set; } = new List<Workspace>();

    private MonitorService _monitorService { get; }
    private ContainerService _containerService;

    public WorkspaceService(ContainerService containerService, MonitorService monitorService)
    {
      _containerService = containerService;
      _monitorService = monitorService;
    }

    /// <summary>
    /// Get active workspaces by iterating over the 2nd level of trees in container forest.
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

    public Workspace GetInactiveWorkspaceByName(string name)
    {
      return InactiveWorkspaces.FirstOrDefault(workspace => workspace.Name == name);
    }

    public Workspace GetWorkspaceFromChildContainer(Container container)
    {
      return container.TraverseUpEnumeration().OfType<Workspace>().FirstOrDefault();
    }

    public Workspace GetFocusedWorkspace()
    {
      var focusedContainer = _containerService.FocusedContainer;

      if (focusedContainer is Workspace)
        return focusedContainer as Workspace;

      return GetWorkspaceFromChildContainer(focusedContainer);
    }
  }
}
