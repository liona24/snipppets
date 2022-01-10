#include <windows.h>
#include <wtsapi32.h>

#include <string.h>


int main(int argc, char* argv[])
{
	BOOL rv;

	char title[] = "At Your Service\0";
	char msg[] = "Hello From Session 0!\0";

	LPSTR pTitle = title;
	LPSTR pMsg = msg;

	if (argc > 2)
		pTitle = argv[2];
	if (argc > 1)
		pMsg = argv[1];

	DWORD titleLen = (DWORD)strlen(pTitle);
	DWORD msgLen = (DWORD)strlen(pMsg);

	PWTS_SESSION_INFO pSessionInfo = NULL;
	DWORD sessionInfoCount = 0;

	rv = WTSEnumerateSessions(
		WTS_CURRENT_SERVER_HANDLE,
		0,
		1,
		&pSessionInfo,
		&sessionInfoCount
	);

	if (!rv)
		return 1;
	
	for (DWORD i = 0; i < sessionInfoCount; ++i) {
		if (pSessionInfo[i].SessionId != (DWORD)0) {

			const DWORD timeout = 0;
			DWORD result = 0;
			const BOOL wait = FALSE;

			WTSSendMessage(
				WTS_CURRENT_SERVER_HANDLE,
				pSessionInfo[i].SessionId,
				pTitle,
				titleLen,
				pMsg,
				msgLen,
				MB_OK,
				timeout,
				&result,
				wait
			);
		}
	}


	WTSFreeMemory(pSessionInfo);

	return 0;
}
