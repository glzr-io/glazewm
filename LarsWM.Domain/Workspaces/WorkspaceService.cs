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

        public Workspace GetWorkspaceFromChildContainer(Container container)
        {
            var parent = container.Parent;

            while (parent != null && parent is Workspace == false)
                parent = parent.Parent;

            return parent as Workspace;
        }

        /// <summary>
        /// Finds workspace that matches given predicate by searching at the 2nd level of container tree.
        /// </summary>
        public Workspace FindWorkspace(Predicate<Workspace> predicate)
        {
            var matchedWorkspace = _containerService.ContainerTree.SelectMany(monitor => monitor.Children).FirstOrDefault((m) =>
            {
                if (predicate(m as Workspace))
                    return true;

                return false;
            });

            return matchedWorkspace as Workspace;
        }

        public Workspace GetWorkspaceByName(string name)
        {
            return FindWorkspace(workspace => workspace.Name == name);
        }
    }
}
