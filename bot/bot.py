import asyncio
import os
import logging
from dotenv import load_dotenv
import aiohttp
from aiogram import Bot, Dispatcher, Router
from aiogram.filters import Command
from aiogram.types import Message

load_dotenv()

BOT_TOKEN = os.environ["BOT_TOKEN"]
HORIZON_URL = os.getenv("HORIZON_URL", "https://horizon-testnet.stellar.org")
LAUNCHPAD_CONTRACT_ID = os.getenv("LAUNCHPAD_CONTRACT_ID", "")
ALERT_CHAT_ID = os.getenv("ALERT_CHAT_ID", "")

logging.basicConfig(level=logging.INFO)
log = logging.getLogger(__name__)

bot = Bot(token=BOT_TOKEN)
dp = Dispatcher()
router = Router()
dp.include_router(router)

MOCK_LAUNCHES = {
    "1": {"name": "LumiToken", "price": 0.05, "progress": 42.0},
    "2": {"name": "StarDrop", "price": 0.12, "progress": 78.5},
}

# Track last seen paging token to avoid duplicate alerts
_last_paging_token: str = "now"


@router.message(Command("start"))
async def cmd_start(msg: Message):
    await msg.answer(
        "👋 *Lumiswap Launch Bot*\n\n"
        "Track token launches on the Lumiswap bonding-curve launchpad.\n\n"
        "Commands:\n"
        "• /launches — list active launches\n"
        "• /price <launch\\_id> — current price for a launch",
        parse_mode="Markdown",
    )


@router.message(Command("launches"))
async def cmd_launches(msg: Message):
    lines = ["📋 *Active Launches*\n"]
    for lid, data in MOCK_LAUNCHES.items():
        lines.append(
            f"#{lid} *{data['name']}* | Price: `{data['price']} XLM` | Progress: `{data['progress']}%`"
        )
    await msg.answer("\n".join(lines), parse_mode="Markdown")


@router.message(Command("price"))
async def cmd_price(msg: Message):
    parts = msg.text.split(maxsplit=1)
    if len(parts) < 2:
        await msg.answer("Usage: /price <launch_id>")
        return
    lid = parts[1].strip()
    launch = MOCK_LAUNCHES.get(lid)
    if not launch:
        await msg.answer(f"No launch found with id `{lid}`.", parse_mode="Markdown")
        return
    await msg.answer(
        f"💰 *{launch['name']}* (#{lid})\n"
        f"Current price: `{launch['price']} XLM`\n"
        f"Progress: `{launch['progress']}%`",
        parse_mode="Markdown",
    )


def _parse_event(record: dict) -> dict | None:
    """Extract LaunchCreated / Migrated data from a Horizon contract event record."""
    try:
        topics = record.get("topic", [])
        if not topics:
            return None
        event_type = topics[0].get("value", "")
        value = record.get("value", {}).get("value", "")
        paging_token = record.get("paging_token", "")
        return {"type": event_type, "value": value, "paging_token": paging_token}
    except Exception:
        return None


async def _poll_events():
    global _last_paging_token
    if not LAUNCHPAD_CONTRACT_ID or not ALERT_CHAT_ID:
        log.warning("LAUNCHPAD_CONTRACT_ID or ALERT_CHAT_ID not set; skipping event polling.")
        return

    url = (
        f"{HORIZON_URL}/contract_events"
        f"?contract_id={LAUNCHPAD_CONTRACT_ID}"
        f"&cursor={_last_paging_token}&order=asc&limit=20"
    )
    try:
        async with aiohttp.ClientSession() as session:
            async with session.get(url, timeout=aiohttp.ClientTimeout(total=10)) as resp:
                if resp.status != 200:
                    log.warning("Horizon returned %s", resp.status)
                    return
                data = await resp.json()
    except Exception as e:
        log.warning("Horizon poll failed: %s", e)
        return

    records = data.get("_embedded", {}).get("records", [])
    for record in records:
        event = _parse_event(record)
        if not event:
            continue
        _last_paging_token = event["paging_token"]

        if event["type"] == "LaunchCreated":
            # value format expected: "<id>:<name>:<target_xlm>"
            parts = event["value"].split(":")
            lid, name, target = (parts + ["?", "?", "?"])[:3]
            text = f"🚀 New Launch #{lid}: {name} | Target: {target} XLM"
        elif event["type"] == "Migrated":
            # value format expected: "<id>:<xlm_raised>"
            parts = event["value"].split(":")
            lid, xlm_raised = (parts + ["?", "?"])[:2]
            text = f"✅ Migrated #{lid}: {xlm_raised} XLM raised → DEX"
        else:
            continue

        try:
            await bot.send_message(ALERT_CHAT_ID, text)
        except Exception as e:
            log.warning("Failed to send alert: %s", e)


async def polling_loop():
    while True:
        await _poll_events()
        await asyncio.sleep(60)


async def main():
    asyncio.create_task(polling_loop())
    await dp.start_polling(bot)


if __name__ == "__main__":
    asyncio.run(main())
