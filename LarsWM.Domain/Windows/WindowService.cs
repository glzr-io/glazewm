using System;
using System.Collections.Generic;
using System.Linq;

namespace LarsWM.Domain.Windows
{
    public class WindowService
    {
        public List<Window> Windows { get; set; } = new List<Window>();
        public Window FocusedWindow { get; set; } = null;

        public Window GetWindowByHandle(IntPtr handle)
        {
            return Windows.FirstOrDefault(w => w.Hwnd == handle);
        }
    }
}
