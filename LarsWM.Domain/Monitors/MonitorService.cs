using System;
using System.Collections.Generic;
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

        /// <summary>
        /// Get the root level of trees in container forest.
        /// </summary>
        public IEnumerable<Monitor> GetMonitors()
        {
            return _containerService.ContainerTree as IEnumerable<Monitor>;
        }

        public Monitor GetMonitorFromChildContainer(Container container)
        {
            return container.UpwardsTraversal().OfType<Monitor>().First();
        }

        public Monitor GetMonitorFromUnaddedWindow(Window window)
        {
            var screen = Screen.FromHandle(window.Hwnd);

            var matchedMonitor = GetMonitors().FirstOrDefault(m => m.Screen.DeviceName == screen.DeviceName);

            if (matchedMonitor == null)
                return GetMonitors().First();

            return matchedMonitor;
        }
    }
}
