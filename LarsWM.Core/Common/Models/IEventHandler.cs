using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.Common.Models
{
    public interface IEventHandler<TEvent> where TEvent : Event
    {
        void Handle(TEvent @event);
    }
}
