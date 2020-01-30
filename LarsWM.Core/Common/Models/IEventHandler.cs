using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.Common.Models
{
    public interface IEventHandler<in TEvent> where TEvent : IEvent
    {
        void Handle(TEvent @event);
    }
}
