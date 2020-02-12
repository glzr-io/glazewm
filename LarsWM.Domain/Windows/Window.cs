using System;
using System.Diagnostics;

namespace LarsWM.Domain.Windows
{
    public class Window
    {
        public Guid Id = Guid.NewGuid();
        public IntPtr Hwnd { get; set; }

        public Window(IntPtr hwnd)
        {
            Hwnd = hwnd;
        }

        public Process Process => GetProcessOfWindow(this);

        public string ClassName => GetClassNameOfWindow(this);

        public WindowRect Location => GetLocationOfWindow(this);

        public bool CanLayout => !IsWindowCloaked(this) && IsWindowManageable(this);

        public void Remove()
        {
            // Clear from list of windows in AppState
            // Clear from WindowsInWorkspace of workspaces in AppState
        }

        /// <summary>
        /// This window's style flags.
        /// </summary>
        //public WindowStyleFlags Style
        //{
        //    get
        //    {
        //        return unchecked((WindowStyleFlags)GetWindowLongPtr(_hwnd, (int)(GWL.GWL_STYLE)).ToInt64());
        //    }
        //    set
        //    {
        //        SetWindowLong(_hwnd, (int)GWL.GWL_STYLE, (int)value);
        //    }

        //}

        /// <summary>
        /// This window's extended style flags.
        /// </summary>
        //[CLSCompliant(false)]
        //public WindowExStyleFlags ExtendedStyle
        //{
        //    get
        //    {
        //        return unchecked((WindowExStyleFlags)GetWindowLongPtr(_hwnd, (int)(GWL.GWL_EXSTYLE)).ToInt64());
        //    }
        //    set
        //    {
        //        SetWindowLong(_hwnd, (int)GWL.GWL_EXSTYLE, (int)value);
        //    }
        //}
    }
}
