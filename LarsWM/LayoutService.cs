using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace LarsWM
{
    class LayoutService
    {
        public static List<WindowLocation> CalculateInitialLayout(Monitor monitor, List<Window> windows)
        {
            var windowLocations = new List<WindowLocation>();
            var windowCount = windows.Count();

            if (windowCount == 0)
            {
                return windowLocations;
            }

            //var windowWidth = (monitor.Width - UserConfig.UserConfigService.InnerGap) / windows.Count;
            var windowWidth = (monitor.Width) / windowCount;

            var index = 1;
            foreach(var window in windows)
            {
                var newWindowLocation = new WindowLocation(index * windowWidth, 0, windowWidth, monitor.Height);
                windowLocations.Add(newWindowLocation);
                index++;
            }

            return windowLocations;
        }
    }
}
