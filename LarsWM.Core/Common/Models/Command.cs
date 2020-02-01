using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.Common.Models
{
    public class Command
    {
        public string Name { get; set; }

        public Command()
        {
            Name = GetType().Name;
        }
    }
}
