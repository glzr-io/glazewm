using GlazeWM.Domain.Containers;
using System.Collections.Generic;
using System.Linq;

namespace GlazeWM.Domain.Workspaces
{
  public class WorkspaceService
  {
    public List<Workspace> InactiveWorkspaces { get; set; } = new List<Workspace>();

    private ContainerService _containerService;

    public WorkspaceService(ContainerService containerService)
    {
      _containerService = containerService;
    }

    /// <summary>
    /// Get active workspaces by iterating over the 2nd level of trees in container tree.
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
      return container.SelfAndAncestors.OfType<Workspace>().FirstOrDefault();
    }

    public Workspace GetFocusedWorkspace()
    {
      return GetWorkspaceFromChildContainer(_containerService.FocusedContainer);
    }
  }
}
