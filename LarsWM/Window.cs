using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM
{
    public class Window
    {
        public IntPtr Hwnd { get; set; }

        public Window(IntPtr hwnd)
        {
            Hwnd = hwnd;
        }

        /// <summary>
        /// This window's style flags.
        /// </summary>
        public WindowStyleFlags Style
        {
            get
            {
                return unchecked((WindowStyleFlags)GetWindowLongPtr(_hwnd, (int)(GWL.GWL_STYLE)).ToInt64());
            }
            set
            {
                SetWindowLong(_hwnd, (int)GWL.GWL_STYLE, (int)value);
            }

        }

        /// <summary>
        /// This window's extended style flags.
        /// </summary>
        [CLSCompliant(false)]
        public WindowExStyleFlags ExtendedStyle
        {
            get
            {
                return unchecked((WindowExStyleFlags)GetWindowLongPtr(_hwnd, (int)(GWL.GWL_EXSTYLE)).ToInt64());
            }
            set
            {
                SetWindowLong(_hwnd, (int)GWL.GWL_EXSTYLE, (int)value);
            }
        }
    }
}
