using LarsWM.Core.Common.Models;
using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.Common.Services
{
    public interface IBus
    {
        void Invoke<T>(T command) where T : ICommand<object>;
        void RaiseEvent<T>(T @event) where T : IEvent;
        void RegisterCommandHandler<T>();
        void RegisterEventHandler<T>();
    }
}
