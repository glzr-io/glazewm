using LarsWM.Core.WindowsApi.DataTypes;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Text;
using static LarsWM.Core.WindowsApi.WindowsApiFacade;

namespace LarsWM.Core
{
    public class Window
    {
        public IntPtr Hwnd { get; set; }

        public Window(IntPtr hwnd)
        {
            Hwnd = hwnd;
        }

        public Process Process => GetProcessOfWindow(this);

        public string ClassName => GetClassNameOfWindow(this);

        public WindowRect Location => GetLocationOfWindow(this);

        public bool CanLayout => !IsWindowCloaked(this) && IsWindowManageable(this);

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
