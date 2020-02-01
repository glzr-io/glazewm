using LarsWM.Core.Common.Models;
using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.Common.Services
{
    /// <summary>
    /// The event bus
    /// </summary>
    public sealed class Bus : IBus
    {
        private static readonly List<Type> RegisteredCommandHandlers = new List<Type>();
        private static readonly List<Type> RegisteredEventHandlers = new List<Type>();

        /// <summary>
        /// Sends command to appropriate command handler.
        /// </summary>
        public void Invoke<T>(T command) where T : ICommand<object>
        {
            // try-catch here? perhaps commands should always return CommandResponse (has State - failure,success, and Data - any return value from command)?
            throw new NotImplementedException();
        }

        /// <summary>
        /// Sends event to appropriate event handler.
        /// </summary>
        public void RaiseEvent<T>(T @event) where T : IEvent
        {
            throw new NotImplementedException();
        }

        public void RegisterCommandHandler<T>()
        {
            RegisteredCommandHandlers.Add(typeof(T));
        }

        public void RegisterEventHandler<T>()
        {
            RegisteredEventHandlers.Add(typeof(T));
        }
    }
}
