using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM
{
    enum FocusDirection
    {
        Up,
        Down,
        Left,
        Right,
    }

    class WindowManager
    {

    public void ShiftFocusInDirection(FocusDirection direction)
    { }

    public void ShiftFocusToWorkspace(Workspace workspace)
    { }

    public void MoveFocusedWindowToWorkspace(Window window, Workspace workspace)
    { }

    public void MoveFocusedWindowInDirection(Window window, FocusDirection direction)
    { }

    }
}

