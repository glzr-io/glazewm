using LarsWM.Domain.Common.Models;
using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Domain.Tree
{
    class ContainerService
    {
        //Tree<Container> Tree = new Tree<Container>();
        List<Container> Tree = new List<Container>();
    }
    // Workspace should have an orientation (horizontal, vertical), but shouldn't extend SplitContainer
    // Doesn't really matter what monitor is to the left or right, it matters what workspace is
    // Should a workspace have the same height/width as the output? Or height/width - gaps?
}
