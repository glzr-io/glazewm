using LarsWM.Domain.Monitors;
using System;
using System.Collections.Generic;
using System.Linq;

namespace LarsWM.Domain.Workspaces
{
    class WorkspaceService
    {
        public List<Workspace> Workspaces { get; set; } = new List<Workspace>();

        private MonitorService _monitorService { get; }

        public WorkspaceService(MonitorService monitorService)
        {
            _monitorService = monitorService;
        }

        public Workspace GetWorkspaceById(Guid id)
        {
            return Workspaces.FirstOrDefault(m => m.Id == id);
        }

        public Workspace GetWorkspaceByName(string name)
        {
            return Workspaces.FirstOrDefault(m => m.Name == name);
        }

        // TODO: Consider changing to `GetInactiveWorkspaces` if only MonitorAddedHandler needs it.
        public List<Workspace> GetActiveWorkspaces()
        {
            return _monitorService.Monitors.SelectMany(m => m.WorkspacesInMonitor).ToList();
        }
    }
}
