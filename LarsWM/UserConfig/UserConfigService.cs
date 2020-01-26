using System;
using System.Collections.Generic;
using System.Text;

namespace LarsWM.UserConfig
{
    public class UserConfigService
    {
        public static string[] WindowClassesToIgnore = new string[] {
            "TaskManagerWindow",
            "MSCTFIME UI",
            "SHELLDLL_DefView",
            "LockScreenBackstopFrame",
            "Progman",
        };

        public static string[] ProcessNamesToIgnore = new string[] {
            "SearchUI",
            "ShellExperienceHost",
            "LockApp",
            "PeopleExperienceHost",
            "StartMenuExperienceHost",
        };
    }
}
