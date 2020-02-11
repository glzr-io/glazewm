using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Infrastructure.Bussing
{
    public class Event
    {
        public string Name { get; set; }

        public Event()
        {
            Name = GetType().Name;
        }
    }
}
