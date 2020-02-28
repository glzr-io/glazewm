using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Monitors;
using System;
using System.Collections.Generic;
using System.Linq;

namespace LarsWM.Domain.Workspaces
{
    class WorkspaceService
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
            return _containerService.ContainerTree
                .SelectMany(monitor => monitor.Children as IEnumerable<Workspace>);
        }

        public Workspace GetWorkspaceByName(string name)
        {
            return GetActiveWorkspaces().FirstOrDefault(workspace => workspace.Name == name);
        }

        public Workspace GetWorkspaceFromChildContainer(Container container)
        {
            return container.UpwardsTraversal().OfType<Workspace>().First();
        }
    }
}
