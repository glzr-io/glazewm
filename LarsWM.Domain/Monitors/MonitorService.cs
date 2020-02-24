using System;
using System.Linq;
using System.Windows.Forms;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows;

namespace LarsWM.Domain.Monitors
{
    public class MonitorService
    {
        private ContainerService _containerService;

        public MonitorService(ContainerService containerService)
        {
            _containerService = containerService;
        }

        public Monitor GetMonitorFromChildContainer(Container container)
        {
            var parent = container.Parent;

            while (parent != null && parent is Monitor == false)
                parent = container.Parent;

            return parent as Monitor;
        }

        /// <summary>
        /// Finds monitor that matches given predicate by searching at the root level of container tree.
        /// </summary>
        public Monitor FindMonitor(Predicate<Monitor> predicate)
        {
            var matchedMonitor = _containerService.ContainerTree.FirstOrDefault((m) =>
            {
                if (predicate(m as Monitor))
                    return true;

                return false;
            });

            return matchedMonitor as Monitor;
        }

        public Monitor GetMonitorFromUnaddedWindow(Window window)
        {
            var screen = Screen.FromHandle(window.Hwnd);

            var matchedMonitor = FindMonitor(m => m.Screen.DeviceName == screen.DeviceName);

            if (matchedMonitor == null)
                return _containerService.ContainerTree[0] as Monitor;

            return matchedMonitor;
        }

        // Not sure if needed.
        //public Monitor GetMonitorFromWorkspace(Workspace workspace)
        //{
        //    return Monitors.FirstOrDefault(m => m.WorkspacesInMonitor.Contains(workspace));
        //}
    }
}
