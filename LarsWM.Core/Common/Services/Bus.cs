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
        private static readonly IDictionary<Type, Type> RegisteredCommandHandlers = new Dictionary<Type, Type>();
        private static readonly IDictionary<Type, Type> RegisteredEventHandlers = new Dictionary<Type, Type>();

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
            throw new NotImplementedException();
        }

        public void RegisterEventHandler<T>()
        {
            throw new NotImplementedException();
        }
    }
}
