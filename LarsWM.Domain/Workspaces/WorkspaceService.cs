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

        public WorkspaceService(MonitorService monitorService)
        {
            _monitorService = monitorService;
        }

        public Workspace GetWorkspaceById(Guid id)
        {
            return InactiveWorkspaces.FirstOrDefault(m => m.Id == id);
        }

        public Workspace GetWorkspaceByName(string name)
        {
            return InactiveWorkspaces.FirstOrDefault(m => m.Name == name);
        }
    }
}
