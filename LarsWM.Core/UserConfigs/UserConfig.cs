using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.Core.UserConfigs
{
    class UserConfig
    {
        public List<string> WindowClassesToIgnore = new List<string>();

        public List<string> ProcessNamesToIgnore = new List<string>();

        public int InnerGap = 20;
        public int OuterGap = 20;
    }
}
