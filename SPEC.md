#SPEC

This is Twitch Sankagata Manager in twitch
in japanese steaming culture, kinda allow user some join their team match game. 
normally , todo this , they use channel point exchange 
so if we detect, some user exchange that channle point item , then we should add to wait list
this program so , need twitch auth feature well for this. and also , need manaing that list well
in now I think , we allow manual way only. if streamer check some member joined some game , then should clean with some button to remove user from there 
but in here , we should have some feature like this turn on/off feature 

fisrt time priority feature
- if some user exchange their channel point itme first time in day. then that user have priority over other users already have joined more than one time (so , should show in list more front than that have already joined ) well for this we can show some badge for side of user name ? 


I have question , some time , streamer can back purchase of channel point item. so in this case, we should reset but can we detect this event ?


# Tech stack
- This should be like electron ? app. but we should think about this will be added as source at OBS. So we need transparent background at all , also for GUI bar for program should be hidden. but if we think about auto update feature , well, it can be better to deploy to somewhere as web. but if so , it is so hard to build well, because , user should control their sankagata managing with web,  and if use this web as source in obs , obs will have sandbox browser, so kinda cannot reflect their system well ? ...
- so if think like that, I think, just build offline webapp as simple as possible,  so just user use that file as source in obs ? ./.. first, what is best way to do this ? (if we deploy with offline , then it is so hard to update well I think ..)
