using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.Common.Models
{
    public interface ICommandHandler<TCommand> where TCommand : Command
    {
        void Handle(TCommand command);
    }
}
