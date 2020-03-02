using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using LarsWM.Domain.Common.Models;
using LarsWM.Domain.Containers;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure.WindowsApi;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows
{
    public class WindowService
    {
        public Window FocusedWindow { get; set; } = null;

        private ContainerService _containerService;
        private UserConfigService _userConfigService;

        public WindowService(ContainerService containerService, UserConfigService userConfigService)
        {
            _containerService = containerService;
            _userConfigService = userConfigService;
        }

        /// <summary>
        /// Get windows by searching entire container forest for Window containers.
        /// </summary>
        public IEnumerable<Window> GetWindows()
        {
            return _containerService.ContainerTree.TraverseDownEnumeration()
                .OfType<Window>();
        }

        /// <summary>
        /// Get windows within given parent container.
        /// </summary>
        public IEnumerable<Window> GetWindowsOfParentContainer(Container parent)
        {
            return parent.TraverseDownEnumeration()
                .OfType<Window>();
        }

        /// <summary>
        /// Get the id of the process that created the window.
        /// </summary>
        public Process GetProcessOfHandle(IntPtr handle)
        {
            uint processId;
            GetWindowThreadProcessId(handle, out processId);

            try
            {
                return Process.GetProcesses().First(process => process.Id == (int)processId);
            } catch(InvalidOperationException)
            {
                return null;
            }
        }

        /// <summary>
        /// Get the name of the class of the window.
        /// </summary>
        public string GetClassNameOfHandle(IntPtr handle)
        {
            var buffer = new StringBuilder(255);
            GetClassName(handle, buffer, buffer.Capacity + 1);
            return buffer.ToString();
        }


        /// <summary>
        /// Get dimensions of the bounding rectangle of the specified window.
        /// </summary>
        public WindowRect GetLocationOfHandle(IntPtr handle)
        {
            WindowRect rect = new WindowRect();
            GetWindowRect(handle, ref rect);
            return rect;
        }

        public bool HandleHasWindowStyle(IntPtr handle, WS style)
        {
            var styles = unchecked((WS)GetWindowLongPtr(handle, (int)(GWL_STYLE)).ToInt64());

            return (styles & style) != 0;
        }

        public bool HandleHasWindowExStyle(IntPtr handle, WS_EX style)
        {
            var styles = unchecked((WS_EX)GetWindowLongPtr(handle, (int)(GWL_EXSTYLE)).ToInt64());

            return (styles & style) != 0;
        }

        private IntPtr GetWindowLongPtr(IntPtr hWnd, int nIndex)
        {
            if (Environment.Is64BitProcess)
            {
                return GetWindowLongPtr64(hWnd, nIndex);
            }
            else
            {
                return new IntPtr(GetWindowLong32(hWnd, nIndex));
            }
        }

        public bool IsHandleCloaked(IntPtr handle)
        {

            bool isCloaked;
            DwmGetWindowAttribute(handle, DwmWindowAttribute.DWMWA_CLOAKED, out isCloaked, Marshal.SizeOf(typeof(bool)));
            return isCloaked;
        }

        public bool IsWindowManageable(Window window)
        {
            var isApplicationWindow = IsWindowVisible(window.Hwnd)
                && !window.HasWindowStyle(WS.WS_CHILD) && !window.HasWindowExStyle(WS_EX.WS_EX_NOACTIVATE);

            var isCurrentProcess = window.Process.Id == Process.GetCurrentProcess().Id;

            var isExcludedClassName = _userConfigService.UserConfig.WindowClassesToIgnore.Contains(window.ClassName);
            var isExcludedProcessName = _userConfigService.UserConfig.ProcessNamesToIgnore.Contains(window.Process.ProcessName);

            var isShellWindow = window.Hwnd == GetShellWindow();

            if (isApplicationWindow && !isCurrentProcess && !isExcludedClassName && !isExcludedProcessName && !isShellWindow)
            {
                return true;
            }

            return false;
        }

        // TODO: Merge with IsWindowManageable method.
        public bool IsHandleManageable(IntPtr handle)
        {

            if (HandleHasWindowExStyle(handle, WS_EX.WS_EX_TOOLWINDOW) ||
                GetWindow(handle, GW.GW_OWNER) != IntPtr.Zero)
            {
                return false;
            }

            return true;
        }
    }
}
